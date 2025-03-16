output "bucket_name" {
  description = "name of the bucket that was created"
  value       = aws_s3_bucket.website_bucket.id
}

output "ecr_repository_url" {
  description = "ecr repository url where images are hosted"
  value       = aws_ecr_repository.gravi_repo.repository_url
}
