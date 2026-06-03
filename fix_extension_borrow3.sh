#!/bin/bash
# Fix all remaining borrow after move patterns with generic approach

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Generic fix for any .push(varname); pattern followed by info! using varname.extension_id
        perl -i -pe '
            if (/\.push\((\w+)\);/ && !/let \w+_id/ && !/let \w+ =/) {
                $var = $1;
                $line = $_;
                # Only fix if the next line is an info! macro using this variable
                $_ = $line;
            }
        ' "$file"
        
        # Fix specific patterns manually
        perl -i -pe '
            if (/\.push\(assertion\);/ && !/let extension_id/) {
                s/\.push\(assertion\);/let extension_id = assertion.extension_id.clone();\n            .push(assertion);/;
            } elsif (/info!\("Verified ([^"]+) for extension {}", assertion\.extension_id\);/) {
                s/info!\("Verified ([^"]+) for extension {}", assertion\.extension_id\);/info!("Verified $1 for extension {}", extension_id);/;
            }
        ' "$file"
        
        perl -i -pe '
            if (/configurations\.insert\(config\.config_id\.clone\(\), config\);/ && !/let config_id/) {
                s/configurations\.insert\(config\.config_id\.clone\(\), config\);/let config_id = config.config_id.clone();\n        let extension_id = config.extension_id.clone();\n        configurations.insert(config_id.clone(), config);/;
            } elsif (/info!\("Set USB configuration {} for extension {}", config\.config_id, config\.extension_id\);/) {
                s/info!\("Set USB configuration {} for extension {}", config\.config_id, config\.extension_id\);/info!("Set USB configuration {} for extension {}", config_id, extension_id);/;
            }
        ' "$file"
        
        perl -i -pe '
            if (/\.push\(transfer\);/ && !/let extension_id/) {
                s/\.push\(transfer\);/let extension_id = transfer.extension_id.clone();\n            .push(transfer);/;
            } elsif (/info!\("Performed ([^"]+) for extension {}", transfer\.extension_id\);/) {
                s/info!\("Performed ([^"]+) for extension {}", transfer\.extension_id\);/info!("Performed $1 for extension {}", extension_id);/;
            }
        ' "$file"
        
        perl -i -pe '
            if (/\.push\(scan\);/ && !/let tag_id/) {
                s/\.push\(scan\);/let tag_id = scan.tag_id.clone();\n        let extension_id = scan.extension_id.clone();\n            .push(scan);/;
            } elsif (/info!\("Scanned ([^"]+) {} for extension {}", scan\.tag_id, scan\.extension_id\);/) {
                s/info!\("Scanned ([^"]+) {} for extension {}", scan\.tag_id, scan\.extension_id\);/info!("Scanned $1 {} for extension {}", tag_id, extension_id);/;
            }
        ' "$file"
        
        perl -i -pe '
            if (/\.push\(feature_report\);/ && !/let extension_id/) {
                s/\.push\(feature_report\);/let extension_id = feature_report.extension_id.clone();\n            .push(feature_report);/;
            } elsif (/info!\("Sent ([^"]+) for extension {}", feature_report\.extension_id\);/) {
                s/info!\("Sent ([^"]+) for extension {}", feature_report\.extension_id\);/info!("Sent $1 for extension {}", extension_id);/;
            }
        ' "$file"
        
        perl -i -pe '
            if (/\.push\(request\);/ && !/let extension_id/) {
                s/\.push\(request\);/let extension_id = request.extension_id.clone();\n            .push(request);/;
            } elsif (/info!\("Requested ([^"]+) for extension {}", request\.extension_id\);/) {
                s/info!\("Requested ([^"]+) for extension {}", request\.extension_id\);/info!("Requested $1 for extension {}", extension_id);/;
            }
        ' "$file"
        
        # Fix unused variable warning
        perl -i -pe 's/if let Some\(entry\) = entries\.remove\(&entry_id\)/if let Some(_entry) = entries.remove(\&entry_id)/' "$file"
    fi
done

echo "Done!"
