### MonoMerger

- Merge N repos
- Get full git trees with authors and times
- Take all the main/master branches start from there
- Go up always commiting the next commit in time

git --no-pager log --all --format="%H;%P;%an;%ae;%at;%cn;%ce;%d;%s" > log.txt

git --no-pager log --all --format="%S;%H;%P;%an;%ae;%at;%cn;%ce;%d;%s" --encoding=UTF-8