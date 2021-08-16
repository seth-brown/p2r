use anyhow::Result;
use chrono::DateTime;
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

const PB_ENDPOINT: &str = "https://api.pinboard.in/v1";

#[derive(Deserialize, Debug, Clone)]
struct PinboardBookmark {
    #[serde(rename(deserialize = "href"))]
    url: String,
    #[serde(rename(deserialize = "description"))]
    title: String,
    #[serde(rename(deserialize = "time"))]
    created: String,
    #[serde(rename(deserialize = "extended"))]
    description: String,
    tags: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct RaindropBookmark {
    url: String,
    folder: String,
    title: String,
    description: String,
    tags: String,
    created: String, // ISO 8601 format
}

struct TransformProps<'a> {
    pinboard_token: &'a str,
    raindrop_folder: &'a str,
    user_tags: &'a Option<String>,
    clean_description: &'a bool,
}

impl PinboardBookmark {
    // add a unique tag to signify a pinboard import
    fn tag(tags: &str, user_tags: &Option<String>) -> String {
        match user_tags {
            None => tags.to_string(),
            Some(t) => [tags, t.as_ref()].join(" "),
        }
    }

    fn clean_description(desc: &str) -> String {
        desc.lines()
            .map(|d| d.trim())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn into_raindrop(
        self,
        folder: &str,
        user_tags: &Option<String>,
        clean_description: &bool,
    ) -> Result<RaindropBookmark> {
        // validate ISO dates
        let _datetime = DateTime::parse_from_rfc3339(&self.created)?;
        let description = match clean_description {
            true => Self::clean_description(&self.description),
            false => self.description,
        };

        let bm = RaindropBookmark {
            url: self.url,
            folder: folder.to_string(),
            title: self.title,
            description,
            tags: Self::tag(&self.tags, user_tags),
            created: self.created,
        };
        Ok(bm)
    }
}

fn write_file(output: PathBuf, data: Vec<RaindropBookmark>) -> Result<()> {
    let mut wtr = Writer::from_path(output)?;
    for datum in data {
        wtr.serialize(datum)?;
    }
    wtr.flush()?;
    Ok(())
}

/// display the number of successfully processed bookmarks and errors
fn stats(n_ok: usize, n_error: usize) {
    let success_msg = format!("✓ {} bookmarks successfully processed", n_ok);
    let error_msg = format!("✕ {} bookmark processing errors ", n_error);
    println!("{}", success_msg);
    println!("{}", error_msg);
}

#[derive(StructOpt, Debug)]
#[structopt(name = "pinboard-to-raindrop")]
struct Opt {
    /// API token:eg. "johndoe:xxx...";
    #[structopt(short, long)]
    pinboard_token: String,

    /// output file with raindrop formatted bookmarks
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// target location in Raindrop for the uploaded bookmarks
    #[structopt(short, long, default_value = "Pinboard Imports")]
    raindrop_folder: String,

    /// append tags to all bookmarks. useful for tagging imported data.
    #[structopt(short, long)]
    user_tags: Option<String>,

    /// clean up descriptions by removing linebreaks
    #[structopt(short, long)]
    clean_description: bool,
}

async fn pb_fetch(url: String) -> Result<Vec<PinboardBookmark>> {
    let req = reqwest::get(url).await?;
    let status = req.status().as_u16();
    match status {
        200 => Ok(req.json::<Vec<PinboardBookmark>>().await?),
        500 => Err(anyhow::anyhow!(
            "HTTP 500: The Pinboard API token may be invalid"
        )),
        s => Err(anyhow::anyhow!(
            "HTTP {}: Unknown error with the Pinboard API",
            s
        )),
    }
}

async fn raindrop(
    p: TransformProps<'_>,
) -> Result<Vec<Result<RaindropBookmark>>> {
    let TransformProps {
        pinboard_token,
        raindrop_folder,
        user_tags,
        clean_description,
    } = p;
    let url = format!(
        "{}/posts/all?auth_token={}&format=json",
        PB_ENDPOINT, pinboard_token
    );
    let (valid, errors): (Vec<Result<RaindropBookmark>>, Vec<_>) =
        pb_fetch(url)
            .await?
            .into_iter()
            .map(|bm| {
                bm.into_raindrop(raindrop_folder, user_tags, clean_description)
            })
            .partition(Result::is_ok);

    stats(valid.len(), errors.len());
    Ok(valid)
}

#[tokio::main]
async fn main() -> Result<()> {
    let Opt {
        pinboard_token,
        output,
        raindrop_folder,
        user_tags,
        clean_description,
    } = Opt::from_args();

    let props = TransformProps {
        pinboard_token: &pinboard_token,
        raindrop_folder: &raindrop_folder,
        user_tags: &user_tags,
        clean_description: &clean_description,
    };

    let bms = raindrop(props)
        .await?
        .into_iter()
        .map(|bm| bm.unwrap())
        .collect::<Vec<RaindropBookmark>>();

    write_file(output, bms)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for RaindropBookmark {
        fn eq(&self, comparer: &Self) -> bool {
            self.url == comparer.url
                && self.created == comparer.created
                && self.title == comparer.title
                && self.tags == comparer.tags
        }
    }

    #[tokio::test]
    async fn invalid_pb_token() {
        let url = format!(
            "{}/posts/all?auth_token={}&format=json",
            PB_ENDPOINT, "XYZ"
        );
        let res = pb_fetch(url).await;
        assert_eq!(false, res.is_ok())
    }

    #[test]
    fn valid_to_raindrop() {
        let description = "Fusce ex ligula, auctor et\nvestibulum eu, \r\nporttitor at dolor.\n\r";
        let tags = "a b c";
        let user_tags = "@pinboard";
        let created = "2017-04-03T15:59:39Z";
        let folder = "Imported";

        let pb_bm = PinboardBookmark {
            url: "url".to_string(),
            title: "title".to_string(),
            created: created.to_string(),
            description: description.to_string(),
            tags: tags.to_string(),
        };

        let rd_bm = RaindropBookmark {
            url: "url".to_string(),
            folder: folder.to_string(),
            title: "title".to_string(),
            description: description.to_string(),
            tags: [tags, user_tags].join(" ").to_string(),
            created: created.to_string(),
        };
        let bm = pb_bm
            .into_raindrop(folder, &Some(user_tags.to_string()), &true)
            .unwrap();
        assert_eq!(rd_bm, bm)
    }

    #[test]
    fn invalid_datetime_to_raindrop() {
        let description = "Fusce ex ligula, auctor et\nvestibulum eu, \r\nporttitor at dolor.\n\r";
        let tags = "a b c";
        let user_tags = "@pinboard";
        let created = "2017/04/03";
        let folder = "Imported";

        let pb_bm = PinboardBookmark {
            url: "url".to_string(),
            title: "title".to_string(),
            created: created.to_string(),
            description: description.to_string(),
            tags: tags.to_string(),
        };

        let bm =
            pb_bm.into_raindrop(folder, &Some(user_tags.to_string()), &true);
        assert_eq!(true, bm.is_err())
    }
}
