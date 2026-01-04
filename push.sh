
if [ -z "$1" ]; then
    msg="normal update"
else
    msg="$1" 
fi

git add .
git commit -m "$msg"
git push