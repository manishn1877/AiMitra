# Skills

Skills are reusable operations for AiMitra, similar to slash commands but as standalone modules.

## How Skills Work

Skills are defined as Markdown files with frontmatter metadata, allowing them to:
- Be discovered and listed in the UI
- Accept user input parameters
- Execute backend commands
- Return structured results

## Skill Definition Format

```yaml
---
name: skill-name
title: Skill Title
description: Brief description of what this skill does
version: 1.0.0
author: Your Name
tags: [tag1, tag2]
parameters:
  - name: param1
    type: string
    description: Description of param1
    required: true
    default: "default value"
---

## Description

Markdown content describing the skill...

## Usage

```
/skill-name param1=value
```