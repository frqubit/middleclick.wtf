# middleclick.wtf

This site converts a middle-click on a Discord image embed to a redirect to any URL you want.

## How it works

Apache checks the UserAgent. If it's Discord, it redirects to the image. If not, it redirects to the URL specified in the `redirect` query parameter.

## Why?

This was created simply out of curiosity. I wanted to see if it was possible to do this, and it turns out it is.

## Hosting yourself

I hosted this a few years ago, but it kept getting horrendous pictures uploaded and I didn't wanna handle that. You can run it using the steps below though:

1. Install Ansible, Terraform, Docker, Node, Cargo, and rsync
2. Change the "domain" variable in `terraform/vars/prod.tfvars` file and `ansible/vars/prod.yml` file to the domain you'd like to host to
3. Run `make`. If the rsync commands fail at the end, try running `make` again
4. It should be fully deployed at this point. To take it down, run `make clean`.
