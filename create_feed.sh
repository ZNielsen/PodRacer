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
args="--data-urlencode \"url=${url}\"" \
    "--data-urlencode \"rate=${rate}\"" \
    "--data-urlencode \"integrate_new=${integrate_new}\""
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

dprint "curl -v -X ${request} -G ${args} ${hostname}:${port}/${slug}"
curl -v -X ${request} -G ${args} ${hostname}:${port}/${slug}


function get_parameters() {
    echo -n "What's the podcast url?: "
    read input_url
    echo -n "What rate would you like the feed to progress at?: "
    read input_rate
    echo -n "Do you want to integrate new episodes to the PodRacer feed? [y/N]: "
    read input_int_new

    # Validate input
    regex='(https?|ftp|file)://[-A-Za-z0-9\+&@#/%?=~_|!:,.;]*[-A-Za-z0-9\+&@#/%=~_|]'
    if [[ $input_url =~ $regex ]]; then
        dprint "Valid input url. Changing to $input_url"
        url="$input_url"
    else
        dprint "Invald input url, sticking with deault: $url"
    fi

    regex='^[0-9]+([.][0-9]+)?$'
    if [[ $input_rate =~ $regex ]]; then
        dprint "Valid input rate. Changing to $input_rate"
        rate="$input_rate"
    else
        dprint "Invald input rate, sticking with deault: $rate"
    fi

    regex='[yY][eE]?[sS]?'
    if [[ $input_int_new =~ $regex ]]; then
        dprint "Valid input integrate_new. Changing to $input_int_new"
        integrate_new="$input_int_new"
    else
        dprint "Invald input integrate_new, sticking with deault: $integrate_new"
    fi
}
