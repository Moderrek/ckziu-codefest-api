use scraper::{Html, Selector};

use crate::models::CkziuNews;

pub async fn async_scrap_cez_news() -> Result<Vec<CkziuNews>, Box<dyn std::error::Error>> {
  let response = reqwest::get("https://cez.lodz.pl").await?;
  let html_content = response.text().await?;
  let document = Html::parse_document(&html_content);
  let news_selector = Selector::parse("div.event-post")?;
  let all_news = document.select(&news_selector);

  let mut parsed_news: Vec<CkziuNews> = Vec::new();

  for news in all_news {
    let a = news.select(&Selector::parse("a")?).next().unwrap();

    let url: String = a.value().attr("href").unwrap().into();
    let title: String = a.text().next().unwrap().into();

    let p = news.select(&Selector::parse("p")?).next().unwrap();
    let description: String = p.text().next().unwrap().into();

    parsed_news.push(CkziuNews {
      title,
      url,
      description,
    });
  }

  Ok(parsed_news)
}
