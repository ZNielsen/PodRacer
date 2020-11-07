#!/bin/bash

debug="true"
function dprint() {
    [[ "$debug" == "true" ]] && echo "$@"
}
dprint "debugging is on"

hostname="http://localhost"
port="8000"
slug="create_feed"

# Debug defaults
url="https://atp.fm/rss"
# rate=1.2
rate=60480 # ten seconds is one week
integrate_new="false"

# Get parameters
# echo -n "What's the podcast url?: "
# read input_url
# echo -n "What rate would you like the feed to progress at?: "
# read input_rate
# echo -n "Do you want to integrate new episodes to the PodRacer feed? [y/N]: "
# read input_int_new

#
# Validate input
#
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


curl -X POST -G \
    --data-urlencode "url=${url}" \
    --data-urlencode "rate=${rate}" \
    --data-urlencode "integrate_new=${integrate_new}" \
    ${hostname}:${port}/${slug}
