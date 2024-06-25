terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "us-west-2"
}

# Create an IAM user for Route 53
resource "aws_iam_user" "route53" {
  name = "route53"
}

variable  hosted_zone_id {
  type = string
  default = "Z03309493AGZOVY2IU47X"
}

# Inline IAM policy for Route 53
resource "aws_iam_policy" "route53" {
  name        = "route53"
  description = "route53"
  policy      = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = [
          "route53:GetChange",
          "route53:ListHostedZonesByName",
          "route53:ListResourceRecordSets"
        ]
        Resource = "*"
      },
      {
        Effect   = "Allow"
        Action   = [
          "route53:ChangeResourceRecordSets"
        ]
        Resource = "arn:aws:route53:::hostedzone/${var.hosted_zone_id}"
      }
    ]
  })
}

# Attach the policy to the user
resource "aws_iam_user_policy_attachment" "route53" {
  user       = aws_iam_user.route53.name
  policy_arn = aws_iam_policy.route53.arn
}

# access key for the user
resource "aws_iam_access_key" "route53" {
  user = aws_iam_user.route53.name
}

output "access_ke_id" {
  value = aws_iam_access_key.route53.id
  sensitive = true
}

output "secret_acceess_key" {
    value = aws_iam_access_key.route53.secret
    sensitive = true
}


