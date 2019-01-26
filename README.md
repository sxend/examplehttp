

prepare

```
curl -sL https://github.com/shyiko/jabba/raw/master/install.sh | bash && . ~/.jabba/jabba.sh
jabba install adopt-openj9@1.8.192-12
jabba alias default adopt-openj9@1.8.192-12

curl https://bintray.com/sbt/rpm/rpm > bintray-sbt-rpm.repo
sudo mv bintray-sbt-rpm.repo /etc/yum.repos.d/
sudo yum install -y sbt git

git clone https://github.com/sxend/examplehttp.git
```

```
cargo run --example main -- --port=8000
```