use docker_tags::Image;

#[tokio::test]
async fn test_docker_hub_existing_image() {
    let image = Image::try_from("nginx").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 1000);

    let image = Image::try_from("docker.io/nginx").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 1000);
}

#[tokio::test]
async fn test_docker_hub_existing_image_with_namespace() {
    let image = Image::try_from("prom/prometheus").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 300);

    let image = Image::try_from("docker.io/prom/prometheus").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 300);
}

#[tokio::test]
async fn test_docker_hub_nonexisting_image() {
    let image = Image::try_from("nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");

    let image = Image::try_from("docker.io/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");
}

#[tokio::test]
async fn test_docker_hub_nonexisting_image_with_namespace() {
    let image = Image::try_from("prom/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");

    let image = Image::try_from("docker.io/prom/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");
}

#[tokio::test]
async fn test_ghcr_existing_image() {
    let image = Image::try_from("ghcr.io/xtls/xray-core").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 1000);
}

#[tokio::test]
async fn test_ghcr_nonexisting_image() {
    let image = Image::try_from("ghcr.io/xtls/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");
}

#[tokio::test]
async fn test_quay_existing_image() {
    let image = Image::try_from("quay.io/prometheus/prometheus").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 300);
}

#[tokio::test]
async fn test_quay_nonexisting_image() {
    let image = Image::try_from("quay.io/prometheus/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");
}

#[tokio::test]
async fn test_angie_existing_image() {
    let image = Image::try_from("docker.angie.software/angie").unwrap();
    let tags = image.fetch_tags().await.unwrap();
    assert!(tags.len() > 200);
}

#[tokio::test]
async fn test_angie_nonexisting_image() {
    let image = Image::try_from("docker.angie.software/nonexistingimage").unwrap();
    let err = image.fetch_tags().await.unwrap_err();
    assert_eq!(err.to_string(), "Image not found");
}
