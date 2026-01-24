# PACS Demo Recording
#
# Generate the demo GIF:
#   vhs demo.tape
#
# VHS:
#   https://github.com/charmbracelet/vhs

Output demo.gif

Require pacs

Set Shell "bash"
Set FontSize 18
Set Width 1200
Set Height 600
Set Padding 20
Set TypingSpeed 80ms
Set Theme "Catppuccin Mocha"

Type "# PACS - Project Aware Command Storage" Sleep 300ms Enter
Sleep 800ms

Type "# Initialize pacs" Sleep 300ms Enter
Type "pacs init" Sleep 300ms Enter Sleep 300ms
Type "MyProject" Sleep 300ms Enter  # Enter project name
Sleep 800ms

Type@80ms "# Add commands to the active project" Sleep 300ms Enter
Type "pacs add hello 'echo Hello World!'" Sleep 300ms Enter
Sleep 800ms

Type "pacs add -t rs build 'cargo build --release'" Sleep 300ms Enter
Sleep 800ms

Type@80ms "# List all commands" Sleep 300ms Enter
Type "pacs ls" Sleep 300ms Enter
Sleep 1.5s

Type@80ms "# Add more commands" Sleep 300ms Enter
Type "pacs add -t k8s deploy 'kubectl apply -f deploy.yaml'" Sleep 300ms Enter
Sleep 800ms

Type@80ms "# List commands in active project" Sleep 300ms Enter
Type "pacs ls" Sleep 300ms Enter
Sleep 2s

Type@80ms "# Add a command with placeholders" Sleep 300ms Enter
Type "pacs add -t k8s pods 'kubectl --context project-{{env}} get pods'" Sleep 300ms Enter
Sleep 800ms

Type@80ms "# Create different environments for the project" Sleep 300ms Enter
Type "pacs env add dev" Sleep 300ms Enter
Sleep 400ms

Type "pacs env add prd" Sleep 300ms Enter
Sleep 800ms

Type@80ms "# Edit the environments (opens editor)" Sleep 300ms Enter
Type "pacs env edit" Sleep 300ms Enter
Sleep 1.2s

# Neovim actions
Type "jjo"
Sleep 200ms
Type 'env = "dev"'
Sleep 400ms
Escape
Sleep 200ms
Type ":wq" Enter
Sleep 1s

Type@80ms "# Switch to an environment" Sleep 300ms Enter
Type "pacs env switch dev" Sleep 300ms Enter
Sleep 800ms

Type "pacs env ls" Sleep 300ms Enter
Sleep 1.5s

Type@80ms "# Show commands with environment values expanded" Sleep 300ms Enter
Type "pacs ls" Sleep 300ms Enter
Sleep 1.5s

Type@80ms "# Copy a command to clipboard" Sleep 300ms Enter
Type "pacs cp pods" Sleep 300ms Enter
Sleep 1.5s

Type "# Done!" Sleep 300ms Enter
Sleep 2s
