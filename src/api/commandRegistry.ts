// Dynamic command discovery and registry
import { apiClient } from './client'

export interface CommandDefinition {
  name: string
  module: string
  description: string
  parameters: ParameterDefinition[]
  returnType: string
  deprecated?: boolean
  since?: string
  examples?: CommandExample[]
}

export interface ParameterDefinition {
  name: string
  type: string
  required: boolean
  description: string
  defaultValue?: any
  validation?: ValidationRule
}

export interface ValidationRule {
  pattern?: string
  min?: number
  max?: number
  options?: any[]
  custom?: (value: any) => boolean | string
}

export interface CommandExample {
  title: string
  description: string
  code: string
  parameters: Record<string, any>
  expectedResult?: any
}

export interface CommandModule {
  name: string
  description: string
  commands: CommandDefinition[]
  version: string
  status: 'active' | 'deprecated' | 'experimental'
}

export class CommandRegistry {
  private static instance: CommandRegistry
  private commands = new Map<string, CommandDefinition>()
  private modules = new Map<string, CommandModule>()
  private initialized = false

  private constructor() {}

  static getInstance(): CommandRegistry {
    if (!CommandRegistry.instance) {
      CommandRegistry.instance = new CommandRegistry()
    }
    return CommandRegistry.instance
  }

  /**
   * Initialize the command registry by discovering available commands
   */
  async initialize(): Promise<void> {
    if (this.initialized) return

    try {
      // Load command definitions for all modules
      await this.loadModuleCommands()
      this.initialized = true
      console.log(`Command registry initialized with ${this.commands.size} commands`)
    } catch (error) {
      console.error('Failed to initialize command registry:', error)
      throw error
    }
  }

  /**
   * Get command definition by name
   */
  getCommand(name: string): CommandDefinition | undefined {
    return this.commands.get(name)
  }

  /**
   * Get all commands for a module
   */
  getModuleCommands(moduleName: string): CommandDefinition[] {
    const module = this.modules.get(moduleName)
    return module ? module.commands : []
  }

  /**
   * Get all available modules
   */
  getModules(): CommandModule[] {
    return Array.from(this.modules.values())
  }

  /**
   * Search commands by pattern
   */
  searchCommands(pattern: string): CommandDefinition[] {
    const regex = new RegExp(pattern, 'i')
    return Array.from(this.commands.values()).filter(cmd =>
      regex.test(cmd.name) ||
      regex.test(cmd.description) ||
      regex.test(cmd.module)
    )
  }

  /**
   * Validate command parameters
   */
  validateParameters(commandName: string, parameters: Record<string, any>): ValidationResult {
    const command = this.getCommand(commandName)
    if (!command) {
      return {
        valid: false,
        errors: [`Unknown command: ${commandName}`]
      }
    }

    const errors: string[] = []
    const warnings: string[] = []

    // Check required parameters
    for (const param of command.parameters) {
      if (param.required && !(param.name in parameters)) {
        errors.push(`Missing required parameter: ${param.name}`)
        continue
      }

      const value = parameters[param.name]
      if (value !== undefined) {
        const validationResult = this.validateParameter(param, value)
        if (!validationResult.valid) {
          errors.push(...validationResult.errors)
        }
        warnings.push(...(validationResult.warnings || []))
      }
    }

    // Check for unknown parameters
    for (const paramName in parameters) {
      if (!command.parameters.some(p => p.name === paramName)) {
        warnings.push(`Unknown parameter: ${paramName}`)
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    }
  }

  /**
   * Get command usage help
   */
  getCommandHelp(commandName: string): CommandHelp | undefined {
    const command = this.getCommand(commandName)
    if (!command) return undefined

    return {
      name: command.name,
      description: command.description,
      module: command.module,
      usage: this.generateUsageString(command),
      parameters: command.parameters.map(p => ({
        name: p.name,
        type: p.type,
        required: p.required,
        description: p.description,
        defaultValue: p.defaultValue
      })),
      examples: command.examples || [],
      deprecated: command.deprecated,
      since: command.since
    }
  }

  /**
   * Generate TypeScript type definitions for commands
   */
  generateTypeDefinitions(): string {
    const modules = Array.from(this.modules.values())
    let output = '// Auto-generated command type definitions\n\n'

    for (const module of modules) {
      output += `// ${module.name} Module\n`
      output += `export namespace ${this.toPascalCase(module.name)} {\n`

      for (const command of module.commands) {
        // Generate parameter interface
        if (command.parameters.length > 0) {
          output += `  export interface ${this.toPascalCase(command.name)}Params {\n`
          for (const param of command.parameters) {
            const optional = param.required ? '' : '?'
            output += `    ${param.name}${optional}: ${param.type}\n`
          }
          output += '  }\n\n'
        }

        // Generate command function type
        const paramType = command.parameters.length > 0
          ? `${this.toPascalCase(command.name)}Params`
          : 'Record<string, never>'

        output += `  export type ${this.toPascalCase(command.name)}Command = `
        output += `(params: ${paramType}) => Promise<${command.returnType}>\n\n`
      }

      output += '}\n\n'
    }

    return output
  }

  /**
   * Load command definitions from backend modules
   */
  private async loadModuleCommands(): Promise<void> {
    // Define the known modules based on the backend structure
    const moduleDefinitions: Partial<CommandModule>[] = [
      { name: 'ai', description: 'AI and machine learning operations' },
      { name: 'workspace', description: 'Workspace management and analysis' },
      { name: 'document', description: 'Document processing and management' },
      { name: 'style', description: 'Style analysis and transfer' },
      { name: 'knowledge', description: 'Knowledge management and analysis' },
      { name: 'search', description: 'Search and indexing operations' },
      { name: 'embedding', description: 'Embedding generation and management' },
      { name: 'conversation', description: 'Conversation intelligence' },
      { name: 'content', description: 'Content adaptation and classification' },
      { name: 'relationship', description: 'Document relationship analysis' },
      { name: 'vector', description: 'Vector operations and similarity' },
      { name: 'template', description: 'Template management and generation' },
      { name: 'format_conversion', description: 'Document format conversion' },
      { name: 'deduplication', description: 'Content deduplication' },
      { name: 'progress', description: 'Progress tracking and monitoring' },
      { name: 'health', description: 'System health monitoring' },
      { name: 'backup', description: 'Backup and restore operations' }
    ]

    for (const moduleDef of moduleDefinitions) {
      try {
        // In a real implementation, this would query the backend for available commands
        // For now, we'll create placeholder definitions
        const moduleCommands = await this.loadModuleCommandDefinitions(moduleDef.name!)

        const module: CommandModule = {
          name: moduleDef.name!,
          description: moduleDef.description!,
          commands: moduleCommands,
          version: '1.0.0',
          status: 'active'
        }

        this.modules.set(module.name, module)

        // Add commands to the main registry
        for (const command of moduleCommands) {
          this.commands.set(command.name, command)
        }
      } catch (error) {
        console.warn(`Failed to load commands for module ${moduleDef.name}:`, error)
      }
    }
  }

  /**
   * Load command definitions for a specific module
   */
  private async loadModuleCommandDefinitions(moduleName: string): Promise<CommandDefinition[]> {
    // In a real implementation, this would make API calls to discover commands
    // For now, return placeholder commands based on the module
    const placeholderCommands: CommandDefinition[] = []

    // This would be replaced with actual backend introspection
    switch (moduleName) {
      case 'workspace':
        placeholderCommands.push(
          {
            name: 'analyze_workspace',
            module: moduleName,
            description: 'Analyze workspace structure and provide insights',
            parameters: [
              { name: 'workspace_path', type: 'string', required: true, description: 'Path to workspace' }
            ],
            returnType: 'WorkspaceAnalysis'
          },
          {
            name: 'get_workspace_health',
            module: moduleName,
            description: 'Get workspace health metrics',
            parameters: [
              { name: 'workspace_id', type: 'string', required: true, description: 'Workspace identifier' }
            ],
            returnType: 'WorkspaceHealth'
          }
        )
        break

      case 'document':
        placeholderCommands.push(
          {
            name: 'process_document',
            module: moduleName,
            description: 'Process and analyze a document',
            parameters: [
              { name: 'file_path', type: 'string', required: true, description: 'Path to document' },
              { name: 'options', type: 'ProcessingOptions', required: false, description: 'Processing options' }
            ],
            returnType: 'DocumentProcessingResult'
          }
        )
        break
    }

    return placeholderCommands
  }

  /**
   * Validate a single parameter
   */
  private validateParameter(param: ParameterDefinition, value: any): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Type checking (basic)
    if (!this.isValidType(value, param.type)) {
      errors.push(`Parameter ${param.name} expected type ${param.type}, got ${typeof value}`)
    }

    // Custom validation rules
    if (param.validation) {
      const rule = param.validation

      if (rule.pattern && typeof value === 'string') {
        const regex = new RegExp(rule.pattern)
        if (!regex.test(value)) {
          errors.push(`Parameter ${param.name} does not match pattern ${rule.pattern}`)
        }
      }

      if (rule.min !== undefined && typeof value === 'number') {
        if (value < rule.min) {
          errors.push(`Parameter ${param.name} must be >= ${rule.min}`)
        }
      }

      if (rule.max !== undefined && typeof value === 'number') {
        if (value > rule.max) {
          errors.push(`Parameter ${param.name} must be <= ${rule.max}`)
        }
      }

      if (rule.options && !rule.options.includes(value)) {
        errors.push(`Parameter ${param.name} must be one of: ${rule.options.join(', ')}`)
      }

      if (rule.custom) {
        const result = rule.custom(value)
        if (typeof result === 'string') {
          errors.push(`Parameter ${param.name}: ${result}`)
        } else if (!result) {
          errors.push(`Parameter ${param.name} failed custom validation`)
        }
      }
    }

    return { valid: errors.length === 0, errors, warnings }
  }

  private isValidType(value: any, expectedType: string): boolean {
    switch (expectedType) {
      case 'string': return typeof value === 'string'
      case 'number': return typeof value === 'number'
      case 'boolean': return typeof value === 'boolean'
      case 'object': return typeof value === 'object' && value !== null
      case 'array': return Array.isArray(value)
      default: return true // Unknown types pass validation
    }
  }

  private generateUsageString(command: CommandDefinition): string {
    const requiredParams = command.parameters.filter(p => p.required)
    const optionalParams = command.parameters.filter(p => !p.required)

    let usage = command.name

    if (requiredParams.length > 0) {
      usage += ' ' + requiredParams.map(p => `<${p.name}>`).join(' ')
    }

    if (optionalParams.length > 0) {
      usage += ' ' + optionalParams.map(p => `[${p.name}]`).join(' ')
    }

    return usage
  }

  private toPascalCase(str: string): string {
    return str.replace(/(^\w|_\w)/g, match => match.replace('_', '').toUpperCase())
  }
}

export interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings?: string[]
}

export interface CommandHelp {
  name: string
  description: string
  module: string
  usage: string
  parameters: ParameterInfo[]
  examples: CommandExample[]
  deprecated?: boolean
  since?: string
}

export interface ParameterInfo {
  name: string
  type: string
  required: boolean
  description: string
  defaultValue?: any
}

// Export singleton instance
export const commandRegistry = CommandRegistry.getInstance()