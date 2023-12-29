pub fn is_reddit_image_url(url: &str) -> bool {
    url.starts_with("https://i.redd.it/")
}
