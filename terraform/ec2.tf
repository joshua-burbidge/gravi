# Security Group allowing SSH and HTTP/HTTPS access
resource "aws_security_group" "web_sg" {
  name        = "web-sg"
  description = "Allow SSH, HTTP, and HTTPS traffic"
  vpc_id      = data.aws_vpc.default_vpc.id

  ingress {
    description = "SSH access"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["207.45.84.101/32", "207.45.84.102/32"]
    # cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTP access from anywhere"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTPS access from anywhere"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "Allow all outbound traffic"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "web-sg"
  }
}

data "aws_ssm_parameter" "amazon_linux_ami" {
  name = "/aws/service/ami-amazon-linux-latest/amzn2-ami-hvm-x86_64-gp2"
}

resource "aws_instance" "app_instance" {
  ami                  = data.aws_ssm_parameter.amazon_linux_ami.value
  instance_type        = "t2.micro"
  key_name             = "gravi-instance-key-pair"
  security_groups      = [aws_security_group.web_sg.name]
  iam_instance_profile = aws_iam_instance_profile.ec2_instance_profile.name

  # install docker - images will be pulled after they are uploaded
  user_data = <<-EOF
    #!/bin/bash
    set -e
    # Update the system and install Docker
    yum update -y
    amazon-linux-extras install docker -y
    service docker start
    chkconfig docker on

  EOF

  tags = {
    Name = "gravi-instance"
  }
}

resource "aws_eip" "lb" {
  instance = aws_instance.app_instance.id
  domain   = "vpc"

  tags = {
    Name = "gravi-instance-elastic-ip"
  }
}

# # Authenticate Docker to ECR
# $(aws ecr get-login --no-include-email --region ${data.aws_region.current.name})
# # Pull the image from ECR repository
# docker pull ${aws_ecr_repository.gravi_repo.repository_url}:${var.git_commit_sha}

# docker run -d -p 80:80 ${aws_ecr_repository.gravi_repo.repository_url}:${var.git_commit_sha}
