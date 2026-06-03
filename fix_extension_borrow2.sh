#!/bin/bash
# Fix additional borrow after move patterns

cd "$(dirname "$0")/src-tauri/src"

for file in extension_*.rs; do
    if [ -f "$file" ]; then
        echo "Processing $file..."
        
        # Fix pattern: .push(event); followed by info! using event.extension_id
        perl -i -pe '
            if (/\.push\(event\);/ && !/let extension_id/) {
                $line = $_;
                $line =~ s/\.push\(event\);/let extension_id = event.extension_id.clone();\n            .push(event);/;
                $_ = $line;
            } elsif (/info!\("Added ([^"]+) for extension {}", event\.extension_id\);/) {
                $_ = "        info!(\"Added $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern: .push(message); followed by info! using message.extension_id
        perl -i -pe '
            if (/\.push\(message\);/ && !/let extension_id/) {
                $line = $_;
                $line =~ s/\.push\(message\);/let extension_id = message.extension_id.clone();\n            .push(message);/;
                $_ = $line;
            } elsif (/info!\("Received ([^"]+) for extension {}", message\.extension_id\);/) {
                $_ = "        info!(\"Received $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern: .push(interaction); followed by info! using interaction.extension_id
        perl -i -pe '
            if (/\.push\(interaction\);/ && !/let extension_id/) {
                $line = $_;
                $line =~ s/\.push\(interaction\);/let extension_id = interaction.extension_id.clone();\n            .push(interaction);/;
                $_ = $line;
            } elsif (/info!\("Added ([^"]+) for extension {}", interaction\.extension_id\);/) {
                $_ = "        info!(\"Added $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern: .push(delivery); followed by info! using delivery.extension_id
        perl -i -pe '
            if (/\.push\(delivery\);/ && !/let extension_id/) {
                $line = $_;
                $line =~ s/\.push\(delivery\);/let extension_id = delivery.extension_id.clone();\n            .push(delivery);/;
                $_ = $line;
            } elsif (/info!\("Delivered ([^"]+) for extension {}", delivery\.extension_id\);/) {
                $_ = "        info!(\"Delivered $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
        
        # Fix pattern: .push(report); followed by info! using report.observer_id
        perl -i -pe '
            if (/\.push\(report\);/ && !/let observer_id/) {
                $line = $_;
                $line =~ s/\.push\(report\);/let observer_id = report.observer_id.clone();\n            .push(report);/;
                $_ = $line;
            } elsif (/info!\("Added ([^"]+) for observer {}", report\.observer_id\);/) {
                $_ = "        info!(\"Added $1 for observer {}\", observer_id);\n";
            }
        ' "$file"
        
        # Fix pattern: .push(operation); followed by info! using operation.extension_id
        perl -i -pe '
            if (/\.push\(operation\);/ && !/let extension_id/) {
                $line = $_;
                $line =~ s/\.push\(operation\);/let extension_id = operation.extension_id.clone();\n            .push(operation);/;
                $_ = $line;
            } elsif (/info!\("Performed ([^"]+) for extension {}", operation\.extension_id\);/) {
                $_ = "        info!(\"Performed $1 for extension {}\", extension_id);\n";
            }
        ' "$file"
    fi
done

echo "Done!"
