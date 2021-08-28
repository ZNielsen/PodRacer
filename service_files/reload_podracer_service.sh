#!/bin/bash

systemctl stop podracer
systemctl daemon-reload
systemctl start podracer

