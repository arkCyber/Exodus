#!/bin/bash
# Fix all .push(var) patterns by adding clone()

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix all .push(varname); patterns that are NOT already using clone()
        perl -i -pe 's/\.push\((\w+)\);/.push($1.clone());/g' "$file"
    fi
done

echo "Done!"
