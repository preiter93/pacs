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
Set TypingSpeed 65ms
Set Theme "Dracula"

Type "# PACS - Project Aware Command Storage" Sleep 200ms Enter
Sleep 600ms

Type "# Initialize pacs" Sleep 200ms Enter
Type "pacs init" Sleep 200ms Enter Sleep 200ms
Type "MyProject" Sleep 200ms Enter  # Enter project name
Sleep 600ms

Type@65ms "# Add commands to the active project" Sleep 200ms Enter
Type "pacs add hello 'echo Hello World!'" Sleep 200ms Enter
Sleep 600ms

Type "pacs add -t docker open-db 'docker exec -it db-1 psql -U postgres -d my_db'" Sleep 200ms Enter
Sleep 600ms

Type@65ms "# List all commands" Sleep 200ms Enter
Type "pacs ls" Sleep 200ms Enter
Sleep 1.5s

Type@65ms "# Add a command with placeholders" Sleep 200ms Enter
Type "pacs add -t k8s pods 'kubectl --context project-{{env}} get pods'" Sleep 200ms Enter
Sleep 600ms

Type@65ms "# Create different environments for the project" Sleep 200ms Enter
Type "pacs env add dev" Sleep 200ms Enter
Sleep 400ms

Type "pacs env add prd" Sleep 200ms Enter
Sleep 600ms

Type@65ms "# Edit the environments (opens editor)" Sleep 200ms Enter
Type "pacs env edit" Sleep 200ms Enter
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

Type@65ms "# Switch to an environment" Sleep 200ms Enter
Type "pacs env switch dev" Sleep 200ms Enter
Sleep 600ms

Type "pacs env ls" Sleep 200ms Enter
Sleep 1.5s

Type@65ms "# Show commands with environment values expanded" Sleep 200ms Enter
Type "pacs ls" Sleep 200ms Enter
Sleep 1.5s

Type@65ms "# Copy a command to clipboard" Sleep 200ms Enter
Type "pacs cp pods" Sleep 200ms Enter
Sleep 1.5s

Type@65ms "# Run a command" Sleep 200ms Enter
Type "pacs run hello" Sleep 200ms Enter
Sleep 1.5s

Type "# That's it!" Sleep 200ms Enter
Sleep 2s
