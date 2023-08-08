variable "region" {
  description = "AWS region"
  default     = "us-east-1"
}

variable "availability_zone" {
  description = "AWS availability zone"
  default     = "us-east-1a"
}

variable "name_prefix" {
  description = "Prefix for all resources"
}

variable "inventory_file" {
  description = "Path to the Ansible inventory file"
}