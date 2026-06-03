#!/bin/bash
# Fix borrow after move errors in extension files

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix pattern: configs.insert(config.extension_id.clone(), config); followed by info using config.extension_id
        sed -i '' '/configs\.insert(config\.extension_id\.clone(), config);/ {
            N
            s/configs\.insert(config\.extension_id\.clone(), config);\n        info!("\([^"]*\) {}", config\.extension_id);/let extension_id = config.extension_id.clone();\n        configs.insert(extension_id.clone(), config);\n        info!("\1 {}", extension_id);/
        }' "$file"
        
        # Fix pattern: permissions.insert(permission.extension_id.clone(), permission); followed by info using permission.extension_id
        sed -i '' '/permissions\.insert(permission\.extension_id\.clone(), permission);/ {
            N
            s/permissions\.insert(permission\.extension_id\.clone(), permission);\n        info!("\([^"]*\) {}", permission\.extension_id);/let extension_id = permission.extension_id.clone();\n        permissions.insert(extension_id.clone(), permission);\n        info!("\1 {}", extension_id);/
        }' "$file"
        
        # Fix pattern: .push(result); followed by info using result.extension_id
        sed -i '' '/\.push(result);/ {
            N
            s/\.or_insert_with(Vec::new)\n            \.push(result);\n\n        info!("\([^"]*\) {}", result\.extension_id);/let extension_id = result.extension_id.clone();\n            .or_insert_with(Vec::new)\n            .push(result);\n\n        info!("\1 {}", extension_id);/
        }' "$file"
    fi
done

echo "Done!"
