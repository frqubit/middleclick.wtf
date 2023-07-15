resource "aws_vpc" "main" {
  cidr_block = "10.15.0.0/16"

  tags = {
    name = "middleclick.wtf-vpc"
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    name = "middleclick.wtf-igw"
  }
}

resource "aws_subnet" "public" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.15.0.0/20"

  tags = {
    name = "middleclick.wtf-public-subnet"
  }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    name = "middleclick.wtf-public-route-table"
  }
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

resource "aws_security_group" "webserver" {
  name        = "middleclick.wtf-webserver-sg"
  description = "Allow HTTP(S) and SSH inbound traffic"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "HTTP"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTPS"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "SSH"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_key_pair" "main" {
  key_name   = "middleclick.wtf-key"
  public_key = file("~/.ssh/id_rsa.pub")
}

data "aws_ami" "debian" {
  most_recent = true

  filter {
    name   = "name"
    values = ["debian-11-amd64-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

resource "aws_network_interface" "webserver" {
  subnet_id         = aws_subnet.public.id
  security_groups   = [aws_security_group.webserver.id]
  private_ips_count = 1
}

resource "aws_instance" "webserver" {
  ami           = data.aws_ami.debian.id
  instance_type = "t2.micro"
  key_name      = aws_key_pair.main.key_name

  network_interface {
    device_index         = 0
    network_interface_id = aws_network_interface.webserver.id
  }

  root_block_device {
    volume_size = 16
    volume_type = "gp3"
  }

  tags = {
    name = "middleclick.wtf-webserver"
  }
}

resource "aws_eip" "webserver" {
  instance = aws_instance.webserver.id

  tags = {
    name = "middleclick.wtf-webserver-eip"
  }
}

resource "aws_eip_association" "webserver" {
  instance_id   = aws_instance.webserver.id
  allocation_id = aws_eip.webserver.id
}

resource "local_file" "ansible_inventory" {
  filename = "inventory.yml"
  content = yamlencode({
    all = {
      hosts = {
        webserver = {
          ansible_host = aws_eip.webserver.public_ip
          ansible_user = "admin"
        }
      }
    }
  })
}

output "ip" {
  value = aws_eip.webserver.public_ip
}
