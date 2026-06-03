#!/bin/bash
# Batch fix borrow after move errors in extension files

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix pattern 1: configs.insert(config.extension_id.clone(), config);
        # followed by info! using config.extension_id
        perl -i -pe '
            if (/configs\.insert\(config\.extension_id\.clone\(\), config\);/) {
                $line = $_;
                $line =~ s/configs\.insert\(config\.extension_id\.clone\(\), config\);/let extension_id = config.extension_id.clone();\n        configs.insert(extension_id.clone(), config);/;
                $_ = $line;
            } elsif (/info!\("Set ([^"]+) config for extension {}", config\.extension_id\);/) {
                $_ = "        info!(\"Set $1 config for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern 2: permissions.insert(permission.extension_id.clone(), permission);
        # followed by info! using permission.extension_id
        perl -i -pe '
            if (/permissions\.insert\(permission\.extension_id\.clone\(\), permission\);/) {
                $line = $_;
                $line =~ s/permissions\.insert\(permission\.extension_id\.clone\(\), permission\);/let extension_id = permission.extension_id.clone();\n        permissions.insert(extension_id.clone(), permission);/;
                $_ = $line;
            } elsif (/info!\("Set ([^"]+) permission for extension {}", permission\.extension_id\);/) {
                $_ = "        info!(\"Set $1 permission for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern 3: .entry(result.extension_id.clone())
        # followed by .push(result); and info! using result.extension_id
        perl -i -pe '
            if (/\.entry\(result\.extension_id\.clone\(\)\)/) {
                $line = $_;
                $line =~ s/\.entry\(result\.extension_id\.clone\(\)\)/let extension_id = result.extension_id.clone();\n        .entry(extension_id.clone())/;
                $_ = $line;
            } elsif (/info!\("Added ([^"]+) for extension {}", result\.extension_id\);/) {
                $_ = "        info!(\"Added $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
    fi
done

echo "Done!"
