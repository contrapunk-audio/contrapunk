#!/bin/bash
# HN Rank Monitor for Contrapunk
# Tracks the position of the HN post over time
# Usage: ./scripts/hn-rank-monitor.sh [interval_seconds]
#
# Data is stored in .hn-monitoring/rank-history.csv (gitignored)

HN_POST_ID="47645025"
INTERVAL="${1:-60}"
DATA_DIR=".hn-monitoring"
CSV_FILE="$DATA_DIR/rank-history.csv"

mkdir -p "$DATA_DIR"

# Create CSV header if it doesn't exist
if [ ! -f "$CSV_FILE" ]; then
    echo "timestamp,rank,points,comments" > "$CSV_FILE"
fi

echo "Monitoring HN post $HN_POST_ID every ${INTERVAL}s"
echo "Data → $CSV_FILE"
echo "Press Ctrl+C to stop"
echo ""

while true; do
    # Fetch the front page and find our post's rank
    RANK=$(curl -s 'https://hacker-news.firebaseio.com/v0/topstories.json' | \
        python3 -c "
import sys, json
ids = json.load(sys.stdin)
try:
    pos = ids.index($HN_POST_ID) + 1
    print(pos)
except ValueError:
    print('off')
" 2>/dev/null)

    # Fetch post details for points and comments
    POST_DATA=$(curl -s "https://hacker-news.firebaseio.com/v0/item/$HN_POST_ID.json")
    POINTS=$(echo "$POST_DATA" | python3 -c "import sys,json; print(json.load(sys.stdin).get('score','?'))" 2>/dev/null)
    COMMENTS=$(echo "$POST_DATA" | python3 -c "import sys,json; print(json.load(sys.stdin).get('descendants','?'))" 2>/dev/null)

    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

    # Append to CSV
    echo "$TIMESTAMP,$RANK,$POINTS,$COMMENTS" >> "$CSV_FILE"

    # Print to terminal
    if [ "$RANK" = "off" ]; then
        echo "[$TIMESTAMP] OFF front page | ${POINTS} pts | ${COMMENTS} comments"
    else
        echo "[$TIMESTAMP] Rank #${RANK} | ${POINTS} pts | ${COMMENTS} comments"
    fi

    sleep "$INTERVAL"
done
