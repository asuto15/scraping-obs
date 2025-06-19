export GITHUB_TOKEN="********"

OWNER="asuto15"
REPO="scraping-obs"

curl -s \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  "https://api.github.com/repos/$OWNER/$REPO/actions/artifacts?per_page=1000" \
| jq -r '.artifacts[].id' \
| while read id; do
    echo "Deleting artifact $id …"
    curl -s \
      -X DELETE \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "Accept: application/vnd.github+json" \
      "https://api.github.com/repos/$OWNER/$REPO/actions/artifacts/$id"
  done
