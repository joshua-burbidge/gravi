resource "aws_s3_bucket" "website_bucket" {
  bucket = "gravi-bucket"
}

resource "aws_s3_bucket_website_configuration" "website_configuration" {
  bucket = aws_s3_bucket.website_bucket.id

  index_document {
    suffix = "index.html"
  }
  error_document {
    key = "error.html"
  }
}

# disabling the default "block public access" settings so that the static website is accessible
resource "aws_s3_bucket_ownership_controls" "controls" {
  bucket = aws_s3_bucket.website_bucket.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_public_access_block" "public_access" {
  bucket = aws_s3_bucket.website_bucket.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_acl" "example" {
  depends_on = [
    aws_s3_bucket_ownership_controls.controls,
    aws_s3_bucket_public_access_block.public_access,
  ]

  bucket = aws_s3_bucket.website_bucket.id
  acl    = "public-read"
}
# ----------------------------------------------------------------

# add a bucket policy that allows everyone to read
resource "aws_s3_bucket_policy" "allow_public_read_access" {
  bucket = aws_s3_bucket.website_bucket.id
  policy = data.aws_iam_policy_document.allow_public_read_access.json
}

data "aws_iam_policy_document" "allow_public_read_access" {
  statement {
    principals {
      type        = "*"
      identifiers = ["*"]
    }

    actions = [
      "s3:GetObject"
    ]

    resources = [
      "${aws_s3_bucket.website_bucket.arn}/*",
    ]
  }
}
# ----------------------------------------------------------
