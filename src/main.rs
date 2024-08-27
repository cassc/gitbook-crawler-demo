use clap::Parser;
use eyre::{ContextCompat, Result};
use playwright::api::Playwright;
use std::path::PathBuf;
use tokio::fs;

#[derive(Parser, Debug)]
#[command(name = "gitbook-crawler")]
#[command(about = "A simple gitbook crawler using Playwright in Rust", long_about = None)]
struct Cli {
    /// The root URL to start crawling from
    url: String,

    /// Path to the browser executable file (default: None)
    #[arg(short, long)]
    executable: Option<PathBuf>,

    /// Directory to save the output
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Headless mode (default: true)
    #[arg(long, default_value_t = true)]
    headless: bool,

    /// Ignore external link (default: true)
    #[arg(long, default_value_t = true)]
    ignore_external_links: bool,
}

/// A struct to represent the content of a page
struct PageContent {
    pub title: String,
    pub link: String,
    pub content: Option<String>,
    pub children: Vec<PageContent>,
}

async fn run_crawler(args: Cli) -> Result<()> {
    // Initialize Playwright and start a Chromium browser
    let playwright = Playwright::initialize().await?;

    let chromimum = playwright.chromium();
    let mut launcher = chromimum.launcher().headless(args.headless);

    if let Some(ref executable) = args.executable {
        launcher = launcher.executable(executable.as_path());
    }

    let browser = launcher.launch().await?;

    // Open a new page in the browser
    let page = browser.context_builder().build().await?.new_page().await?;

    // Navigate to the root URL
    page.goto_builder(&args.url).goto().await?;

    // Extract the page title
    let title = page.title().await?;
    println!("Title: {}", title);

    let mut pages = vec![];

    let content = page
        .query_selector("main")
        .await?
        .context("No main element found")?
        .inner_html()
        .await?;

    let title_page = PageContent {
        title,
        link: "index".to_string(),
        content: Some(content),
        children: vec![],
    };

    pages.push(title_page);

    let aside = page
        .query_selector("aside")
        .await?
        .context("No side panel element found")?;

    // Extract all links (anchor elements)
    let links = aside.query_selector_all("a").await?;

    for link in links.iter() {
        if let Some(href) = link.get_attribute("href").await? {
            let link_text = link.text_content().await?.unwrap_or_default();
            println!("Link text: {} | URL: {}", link_text, href);
            let page_content = PageContent {
                title: link_text,
                link: href,
                content: None,
                children: vec![],
            };
            pages.push(page_content);
        }
    }

    if let Some(output_dir) = args.output_dir {
        for page_content in pages.iter_mut() {
            if &page_content.link == "/" {
                page_content.link = "index".to_string();
            }

            let link = page_content.link.trim_start_matches('/');

            let output_path = output_dir.join(format!("{}.html", link));
            println!("Creating: {:?}", output_path);

            if output_path.exists() {
                continue;
            }
            if page_content.content.is_some() {
                continue;
            }

            let link = &page_content.link;
            let link = if link.starts_with("http") {
                if args.ignore_external_links {
                    continue;
                } else {
                    panic!("External links are not supported")
                }
            } else {
                format!("{}/{}", args.url, link)
            };

            page.goto_builder(&link).goto().await?;
            let content = page
                .query_selector("main")
                .await?
                .context("No main element found")?
                .inner_html()
                .await?;
            page_content.content = Some(content.clone());

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            fs::write(output_path, content).await?;
        }
    }

    // Close the browser
    browser.close().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    run_crawler(args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawl() -> Result<()> {
        let args = Cli {
            url: "https://docs.blueberry.garden/".to_string(),
            output_dir: None,
            headless: true,
            executable: "/usr/bin/chromium".parse::<PathBuf>().ok(),
            ignore_external_links: true,
        };

        // Call the refactored function directly
        run_crawler(args).await?;
        Ok(())
    }
}
