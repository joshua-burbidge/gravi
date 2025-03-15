resource "aws_s3_bucket" "gravi_bucket" {
  bucket = "gravi-bucket"
}

resource "aws_s3_bucket_website_configuration" "example" {
  bucket = aws_s3_bucket.gravi_bucket.id

  index_document {
    suffix = "index.html"
  }
  error_document {
    key = "error.html"
  }
}
