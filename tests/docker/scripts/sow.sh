SPARK_CLIENT_PATH=/Users/jamesyeap/Developer/amps/downloaded_folders/AMPS-5.3.4.69-Release-Linux/bin/spark
HOST=127.0.0.1:9007

$SPARK_CLIENT_PATH sow -server $HOST -proto amps -type json -topic messages
