ENVIRON ?= dev

deploy: build ansible
	rsync ./target/x86_64-unknown-linux-musl/release/api admin@$(shell terraform output ip):/home/admin/api
	rsync -rltz ./public/* admin@$(shell terraform output ip):/var/www/middleclick.wtf
	ssh admin@$(shell terraform output ip) "sudo systemctl enable api"
	ssh admin@$(shell terraform output ip) "sudo systemctl restart api"

init:
	npm install
	cargo install cross
	terraform init -var-file=terraform/vars/${ENVIRON}.tfvars

ansible: terraform
	ansible-playbook -i inventory.yml ansible/playbook.yml --extra-vars "@ansible/vars/${ENVIRON}.yml"

build: init
	cross build -p api --target x86_64-unknown-linux-musl --release
	npm run tailwind

terraform: init
	terraform apply -auto-approve -var-file=terraform/vars/${ENVIRON}.tfvars