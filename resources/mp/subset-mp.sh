#!/bin/bash

# TODO: update path
MP="/path/to/maxo-base.obo"

# TODO: setup Robot by following instructions
# at https://robot.obolibrary.org/
echo ${MP}
module load robot

# MP:0000603 pale liver
robot --input ${MP} --method BOT --term MP:0000603 \
  convert --output pl.mp.obo

# MP:0002224 abnormal spleen size
robot extract --input ${MP} --method BOT --term MP:0002224 \
  convert --output asp.mp.obo

# MP:0005308 abnormal circulating ammonia level
robot extract --input ${MP} --method BOT --term MP:0005308 \
  convert --output acal.mp.obo

# Merge into one file
robot merge --input pl.mp.obo \
  --input asp.mp.obo \
  --input acal.mp.obo \
  --output mp.toy.json

rm *.obo