#!/bin/bash
# Fix all borrow after move errors by adding clone() calls

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix all .insert(var.field.clone(), var); patterns
        perl -i -pe 's/\.insert\((\w+)\.(\w+)\.clone\(\), \1\);/.insert($1.$2.clone(), $1.clone());/g' "$file"
        
        # Fix all .push(var); patterns that are followed by info! using var
        # This is more complex, so we'll handle specific cases
    fi
done

echo "Done!"
