# Security Group allowing SSH and HTTP/HTTPS access
resource "aws_security_group" "web_sg" {
  name        = "web-sg"
  description = "Allow SSH, HTTP, and HTTPS traffic"
  vpc_id      = data.aws_vpc.default_vpc.id

  ingress {
    description = "SSH access from your IP"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["207.45.84.102/32"] # Replace <YOUR_IP> with your public IP
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
output "ami" {
  description = "amazon linux ami"
  value       = data.aws_ssm_parameter.amazon_linux_ami.value
}

# EC2 Instance that runs Docker and deploys a container from ECR
resource "aws_instance" "app_instance" {
  ami                  = data.aws_ssm_parameter.amazon_linux_ami.value
  instance_type        = "t2.micro"
  key_name             = "gravi-instance-key-pair" # Replace with your key pair name
  security_groups      = [aws_security_group.web_sg.name]
  iam_instance_profile = aws_iam_instance_profile.ec2_instance_profile.name

  # User data installs Docker, logs into ECR, pulls and runs your image
  user_data = <<-EOF
    #!/bin/bash
    set -e
    # Update the system and install Docker
    yum update -y
    amazon-linux-extras install docker -y
    service docker start
    chkconfig docker on

    # Authenticate Docker to ECR
    $(aws ecr get-login --no-include-email --region ${data.aws_region.current.name})


  EOF
}

# # Pull the image from ECR repository
# docker pull ${aws_ecr_repository.gravi_repo.repository_url}:${var.git_commit_sha}

# docker run -d -p 80:80 ${aws_ecr_repository.gravi_repo.repository_url}:${var.git_commit_sha}
