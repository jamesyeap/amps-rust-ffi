SPARK_CLIENT_PATH=/Users/jamesyeap/Developer/amps/downloaded_folders/AMPS-5.3.4.69-Release-Linux/bin/spark
HOST=127.0.0.1:9007
FILE=./messages/messages.txt

$SPARK_CLIENT_PATH publish -server $HOST -proto amps -type json -topic messages -file $FILE
