# Advanced Template with Processing DSL

This example demonstrates using the Processing DSL for dynamic templating.

## Template Structure

```
~/.local/masstemplate/node-api/
├── package.json
├── src/
│   ├── app.js
│   └── routes/
│       └── users.js
├── .mtem/
│   ├── config
│   └── post_install.sh
└── README.md
```

## Processing DSL Configuration (.mtem/config)

```dsl
# Replace placeholders with user input
replace __PROJECT_NAME__ {{ project_name }}
replace __PORT__ {{ port | default(3000) }}

# Set environment variables
dotenv set NODE_ENV development
dotenv set PORT {{ port | default(3000) }}

# Handle package.json specially
match package.json {
    replace "my-api" {{ project_name }}
}

# Default collision strategy
collision backup
```

## Interactive Prompts (.mtem/copier.yml)

```yaml
project_name:
  type: str
  help: What is the name of your project?

port:
  type: int
  default: 3000
  help: Which port should the server run on?
```

## Post-Install Script (.mtem/post_install.sh)

```bash
#!/bin/bash
npm install
echo "API server ready! Run 'npm start' to begin development."
```

## Usage

```bash
mtem apply node-api --dest my-api-project
```

The tool will:
1. Prompt for project name and port
2. Copy files with replacements applied
3. Set environment variables
4. Run npm install
5. Display success message