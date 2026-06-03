#!/bin/bash
# Fix remaining borrow after move errors

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix pattern: permissions.insert(key, permission); followed by info! using permission.field
        perl -i -pe '
            if (/permissions\.insert\(key, permission\);/ && !/let extension_id/) {
                s/permissions\.insert\(key, permission\);/let extension_id = permission.extension_id.clone();\n        let sensor_type = permission.sensor_type.clone() if defined permission.sensor_type;\n        let source_type = permission.source_type.clone() if defined permission.source_type;\n        let device_type = permission.device_type.clone() if defined permission.device_type;\n        permissions.insert(key, permission);/;
            }
        ' "$file"
        
        # Fix pattern: task.status = status; followed by info! using status
        perl -i -pe 's/task\.status = status;/task.status = status.clone();/g' "$file"
        
        # Fix double clone() issues
        perl -i -pe 's/\.clone\(\)\.clone\(\)/.clone()/g' "$file"
    fi
done

echo "Done!"
