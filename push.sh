
if [ -z "$1" ]; then
    msg="normal update"
else
    msg="$*" 
fi

git add .
git commit -m "$msg"
git push origin master