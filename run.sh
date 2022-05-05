#!/bin/bash
if [ "$1" == "start" ]
then
   if [ "$2" == "release" ]
   then
      nohup ./target/release/steve > log.txt &
   elif [ "$2" == "debug" ]
   then
      nohup ./target/debug/steve > log.txt &
   fi
elif [ "$1" == "restart" ]
then
   if [ "$2" == "release" ]
   then
      kill -9 $(pgrep -f steve)
      nohup ./target/release/steve > log.txt &
   elif [ "$2" == "debug" ]
   then
      kill -9 $(pgrep -f steve)
      nohup ./target/debug/steve > log.txt &
   fi
elif [ "$1" == "kill" ]
then
   kill -9 $(pgrep -f steve)
elif [ "$1" == "log" ]
then
   docker compose logs | grep cloud
fi
