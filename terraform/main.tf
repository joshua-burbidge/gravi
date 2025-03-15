terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.91"
    }
  }
  backend "s3" {
    bucket = "575737149124-terraform-backend"
    key    = "gravi/tfstate"
    region = "us-east-1"
  }

  required_version = ">= 1.9.2"
}

provider "aws" {
  default_tags {
    tags = {
      app = "gravi"
    }
  }
}

data "aws_caller_identity" "aws_account" {}
