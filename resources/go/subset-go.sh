#!/bin/bash

# TODO: update path
GO="/path/to/go-basic.obo"

# TODO: setup Robot by following instructions
# at https://robot.obolibrary.org/
module load robot

# GO:0051146 striated muscle cell differentiation
robot extract --input ${GO} --method BOT --term GO:0051146 \
  convert --output smcd.hp.obo

# GO:0052693 epoxyqueuosine reductase activity
robot extract --input ${GO} --method BOT --term GO:0052693 \
  convert --output era.hp.obo

# GO:0005634 nucleus
robot extract --input ${GO} --method BOT --term GO:0005634 \
  convert --output n.hp.obo

# Merge into one file
robot merge --input smcd.hp.obo \
  --input era.hp.obo \
  --input n.hp.obo \
  --output go.toy.json

rm *.obo