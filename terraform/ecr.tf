resource "aws_ecr_repository" "gravi_repo" {
  name                 = var.ecr_repository_name
  image_tag_mutability = "IMMUTABLE"
  force_delete         = true
}

resource "aws_ecr_lifecycle_policy" "lifecycle_policy" {
  repository = aws_ecr_repository.gravi_repo.name
  policy     = file("./lifecycle-policy.json")
}

resource "aws_ecr_repository_policy" "policy" {
  repository = aws_ecr_repository.gravi_repo.name
  policy     = data.aws_iam_policy_document.aws_ecr_repository_policy.json
}

data "aws_iam_policy_document" "aws_ecr_repository_policy" {
  statement {
    effect  = "Allow"
    actions = ["ecr:*"]
    principals {
      type = "AWS"
      identifiers = [
        "arn:aws:iam::575737149124:user/admin",
        "arn:aws:iam::575737149124:role/gravi-deploy",
        # aws_lightsail_container_service.container_service.private_registry_access[0].ecr_image_puller_role[0].principal_arn
      ]
    }
  }
}
