#!/bin/bash

debug="true"
function dprint() {
    [[ "$debug" == "true" ]] && echo "$@"
}
dprint "debugging is on"


hostname="http://localhost"
port="42069"
slug="create_feed"

# Debug defaults
url="https://atp.fm/rss"
# rate=1.2
rate=300
integrate_new="false"


request="POST"
# Parse out command line args
shopt -s extglob
while test $# -gt 0; do
    dprint "parsing |$1|"
    case $1 in
        -l|--list-feeds)
            dprint "In list feeds"
            slug="list_feeds"
            request="GET"
            args=""
            shift 1
        ;;
        --get)
            dprint "In get"
            slug="podcasts"
            request="GET"
            args=""
            shift 1
            url="$1/racer.rss"
            shift 1
        ;;
        --update)
            dprint "In update"
            slug="update"
            request="GET"
            args=""
            shift 1
            update_url="$1"
            shift 1
        ;;
        --update-all)
            dprint "In update all"
            slug="update"
            request="GET"
            args=""
            shift 1
        ;;
        -u|--url)
            dprint "In url"
            shift 1
            url="$1"
            shift 1
            args="--data-urlencode \"url=${url}\"" \
                "--data-urlencode \"rate=${rate}\"" \
                "--data-urlencode \"integrate_new=${integrate_new}\""
        ;;
        -r|--rate)
            dprint "In rate"
            shift 1
            rate="$1"
            shift 1
            args="--data-urlencode \"url=${url}\"" \
                "--data-urlencode \"rate=${rate}\"" \
                "--data-urlencode \"integrate_new=${integrate_new}\""
        ;;
        -i|--integrate-new)
            dprint "In integrate new"
            shift 1
            integrate_new="$1"
            shift 1
            args="--data-urlencode \"url=${url}\"" \
                "--data-urlencode \"rate=${rate}\"" \
                "--data-urlencode \"integrate_new=${integrate_new}\""
        ;;
        -p|--port)
            dprint "In port"
            shift 1
            port="$1"
            shift 1

        ;;
        -h|--hostname)
            dprint "In hostname"
            shift 1
            hostname="$1"
            shift 1
        ;;
        *)
            echo "Can't match $0"
            exit 1
        ;;
    esac
done

# if [[ "$pass_args" == "true" ]]; then
    # curl -X ${request} -G \
    #     --data-urlencode "url=${url}" \
    #     --data-urlencode "rate=${rate}" \
    #     --data-urlencode "integrate_new=${integrate_new}" \
    #     ${hostname}:${port}/${slug}
# else
    dprint "curl -v -X ${request} -G ${hostname}:${port}/${slug}"
    curl -v -X ${request} -G ${hostname}:${port}/${slug}
# fi
