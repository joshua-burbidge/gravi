output "bucket_name" {
  description = "name of the bucket that was created"
  value       = aws_s3_bucket.gravi_bucket.id
}
