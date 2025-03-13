#!/bin/bash

# TODO: update path
MAXO="/path/to/maxo-base.obo"

# TODO: setup Robot by following instructions
# at https://robot.obolibrary.org/
module load robot

# MAXO:0000682 cardiologist evaluation
robot extract --input ${MAXO} --method BOT --term MAXO:0000682 \
  convert --output ce.maxo.obo

# MAXO:0000185 antiarrythmic agent therapy
robot extract --input ${MAXO} --method BOT --term MAXO:0000185 \
  convert --output aat.maxo.obo

# MAXO:0035118 cardiac catheterization
robot extract --input ${MAXO} --method BOT --term MAXO:0035118 \
  convert --output cc.maxo.obo

# Merge into one file
robot merge --input ce.maxo.obo \
  --input aat.maxo.obo \
  --input cc.maxo.obo \
  --output maxo.toy.json

rm *.obo