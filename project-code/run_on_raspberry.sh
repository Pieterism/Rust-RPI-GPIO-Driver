first_argument="$1"
echo $first_argument

cargo build --release
scp -r target/release pi@$first_argument:~/binaries
ssh pi@$first_argument <<-'ENDSSH'
	sudo ./binaries/project-code
ENDSSH

