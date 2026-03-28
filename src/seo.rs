use std::path::Path;
use crate::error::{CangkangError, IoContext};
use crate::models::PageInfo;
use crate::log_success;

pub const BASE_URL: &str = "https://siputbiru.me";

pub fn generate_assets(pages: &[PageInfo], dist_dir: &Path) -> Result<(), CangkangError> {
    build_sitemap(pages, dist_dir)?;
    build_rss(pages, dist_dir)?;
    Ok(())
}

fn build_sitemap(pages: &[PageInfo], dist_dir: &Path) -> Result<(), CangkangError> {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    // Add index
    sitemap.push_str(&format!(
        "  <url>\n    <loc>{}/</loc>\n  </url>\n",
        BASE_URL
    ));

    for page in pages {
        sitemap.push_str(&format!(
            "  <url>\n    <loc>{}/{}</loc>\n    <lastmod>{}</lastmod>\n  </url>\n",
            BASE_URL, page.url, page.date
        ));
    }

    sitemap.push_str("</urlset>");

    let sitemap_path = dist_dir.join("sitemap.xml");
    std::fs::write(&sitemap_path, sitemap).with_ctx(sitemap_path.to_string_lossy())?;
    log_success!("Generated sitemap.xml");

    Ok(())
}

fn build_rss(pages: &[PageInfo], dist_dir: &Path) -> Result<(), CangkangError> {
    let mut rss = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n\
        <rss version=\"2.0\">\n\
        <channel>\n\
          <title>SiputBiru's Notes</title>\n\
          <link>{}</link>\n\
          <description>A minimal, dependency-free Static Site Generator</description>\n",
        BASE_URL
    );

    for page in pages {
        let pub_date = if page.date.is_empty() {
            String::new()
        } else {
            format!("    <pubDate>{}</pubDate>\n", format_rfc822(&page.date))
        };

        rss.push_str(&format!(
            "  <item>\n\
                <title>{}</title>\n\
                <link>{}/{}</link>\n\
                <description>{}</description>\n\
            {}  </item>\n",
            page.title, BASE_URL, page.url, page.description, pub_date
        ));
    }

    rss.push_str("</channel>\n</rss>");

    let rss_path = dist_dir.join("index.xml");
    std::fs::write(&rss_path, rss).with_ctx(rss_path.to_string_lossy())?;
    log_success!("Generated index.xml (RSS)");

    Ok(())
}

fn format_rfc822(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }

    let year = parts[0];
    let month_idx = parts[1].parse::<usize>().unwrap_or(0);
    let day = parts[2];

    let months = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let month = if month_idx > 0 && month_idx <= 12 {
        months[month_idx]
    } else {
        "Jan"
    };

    format!("{} {} {} 00:00:00 GMT", day, month, year)
}
