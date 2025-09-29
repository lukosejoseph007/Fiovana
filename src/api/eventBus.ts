// Real-time event handling system
export interface EventSubscriber<T = unknown> {
  id: string
  callback: (data: T) => void | Promise<void>
  options: SubscriptionOptions
  createdAt: Date
  lastTriggered?: Date
}

export interface SubscriptionOptions {
  once?: boolean
  filter?: (data: unknown) => boolean
  priority?: number
  throttle?: number
  debounce?: number
}

export interface Event<T = unknown> {
  id: string
  type: string
  data: T
  timestamp: Date
  source?: string
  metadata?: Record<string, unknown>
}

export interface EventMetrics {
  totalEvents: number
  eventsByType: Record<string, number>
  subscriberCount: number
  averageProcessingTime: number
  lastEventTime?: Date
}

export class EventBus {
  private static instance: EventBus
  private subscribers = new Map<string, Set<EventSubscriber>>()
  private eventHistory: Event[] = []
  private metrics: EventMetrics = {
    totalEvents: 0,
    eventsByType: {},
    subscriberCount: 0,
    averageProcessingTime: 0
  }
  private throttleTimers = new Map<string, number>()
  private debounceTimers = new Map<string, number>()

  private constructor() {}

  static getInstance(): EventBus {
    if (!EventBus.instance) {
      EventBus.instance = new EventBus()
    }
    return EventBus.instance
  }

  /**
   * Subscribe to events of a specific type
   */
  subscribe<T = unknown>(
    eventType: string,
    callback: (data: T) => void | Promise<void>,
    options: SubscriptionOptions = {}
  ): string {
    const subscriber: EventSubscriber<T> = {
      id: this.generateSubscriberId(),
      callback,
      options,
      createdAt: new Date()
    }

    if (!this.subscribers.has(eventType)) {
      this.subscribers.set(eventType, new Set())
    }

    this.subscribers.get(eventType)!.add(subscriber)
    this.metrics.subscriberCount++

    return subscriber.id
  }

  /**
   * Unsubscribe from events
   */
  unsubscribe(subscriptionId: string): boolean {
    for (const [eventType, subscriberSet] of this.subscribers.entries()) {
      for (const subscriber of subscriberSet) {
        if (subscriber.id === subscriptionId) {
          subscriberSet.delete(subscriber)
          this.metrics.subscriberCount--

          // Clean up empty event types
          if (subscriberSet.size === 0) {
            this.subscribers.delete(eventType)
          }

          return true
        }
      }
    }
    return false
  }

  /**
   * Emit an event to all subscribers
   */
  async emit<T = unknown>(
    eventType: string,
    data: T,
    source?: string,
    metadata?: Record<string, unknown>
  ): Promise<void> {
    const event: Event<T> = {
      id: this.generateEventId(),
      type: eventType,
      data,
      timestamp: new Date(),
      source,
      metadata
    }

    // Update metrics
    this.updateMetrics(event)

    // Store in history
    this.addToHistory(event)

    // Get subscribers for this event type
    const subscribers = this.subscribers.get(eventType)
    if (!subscribers || subscribers.size === 0) {
      return
    }

    // Sort subscribers by priority (higher priority first)
    const sortedSubscribers = Array.from(subscribers).sort(
      (a, b) => (b.options.priority || 0) - (a.options.priority || 0)
    )

    // Process subscribers
    const processingPromises: Promise<void>[] = []

    for (const subscriber of sortedSubscribers) {
      // Apply filter if specified
      if (subscriber.options.filter && !subscriber.options.filter(data)) {
        continue
      }

      // Handle throttling
      if (subscriber.options.throttle) {
        const throttleKey = `${eventType}:${subscriber.id}`
        const lastCall = this.throttleTimers.get(throttleKey) || 0
        const now = Date.now()

        if (now - lastCall < subscriber.options.throttle) {
          continue
        }

        this.throttleTimers.set(throttleKey, now)
      }

      // Handle debouncing
      if (subscriber.options.debounce) {
        const debounceKey = `${eventType}:${subscriber.id}`
        const existingTimer = this.debounceTimers.get(debounceKey)

        if (existingTimer) {
          clearTimeout(existingTimer)
        }

        const timer = setTimeout(async () => {
          await this.executeSubscriber(subscriber, event)
          this.debounceTimers.delete(debounceKey)
        }, subscriber.options.debounce)

        this.debounceTimers.set(debounceKey, timer as unknown as number)
        continue
      }

      // Execute subscriber immediately
      processingPromises.push(this.executeSubscriber(subscriber, event))
    }

    // Wait for all subscribers to complete
    await Promise.allSettled(processingPromises)
  }

  /**
   * Get event history
   */
  getEventHistory(eventType?: string, limit?: number): Event[] {
    let events = eventType
      ? this.eventHistory.filter(e => e.type === eventType)
      : this.eventHistory

    if (limit) {
      events = events.slice(-limit)
    }

    return events
  }

  /**
   * Get event metrics
   */
  getMetrics(): EventMetrics {
    return { ...this.metrics }
  }

  /**
   * Clear event history
   */
  clearHistory(): void {
    this.eventHistory = []
  }

  /**
   * Get active subscriptions
   */
  getSubscriptions(): Record<string, number> {
    const subscriptions: Record<string, number> = {}

    for (const [eventType, subscriberSet] of this.subscribers.entries()) {
      subscriptions[eventType] = subscriberSet.size
    }

    return subscriptions
  }

  /**
   * Wait for a specific event (Promise-based)
   */
  waitFor<T = unknown>(
    eventType: string,
    filter?: (data: T) => boolean,
    timeout?: number
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      let subscriptionId: string = ''
      let timeoutId: number | undefined

      const cleanup = () => {
        if (subscriptionId) {
          this.unsubscribe(subscriptionId)
        }
        if (timeoutId) {
          clearTimeout(timeoutId)
        }
      }

      // Set up timeout if specified
      if (timeout) {
        timeoutId = setTimeout(() => {
          cleanup()
          reject(new Error(`Timeout waiting for event: ${eventType}`))
        }, timeout) as unknown as number
      }

      // Subscribe to the event
      subscriptionId = this.subscribe<T>(
        eventType,
        (data: T) => {
          cleanup()
          resolve(data)
        },
        {
          once: true,
          filter
        }
      )
    })
  }

  /**
   * Create a typed event emitter for a specific event type
   */
  createTypedEmitter<T = unknown>(eventType: string) {
    return {
      emit: (data: T, source?: string, metadata?: Record<string, unknown>) =>
        this.emit(eventType, data, source, metadata),

      subscribe: (
        callback: (data: T) => void | Promise<void>,
        options: SubscriptionOptions = {}
      ) => this.subscribe(eventType, callback, options),

      waitFor: (filter?: (data: T) => boolean, timeout?: number) =>
        this.waitFor(eventType, filter, timeout)
    }
  }

  /**
   * Execute a subscriber with error handling and metrics
   */
  private async executeSubscriber<T>(
    subscriber: EventSubscriber<T>,
    event: Event<T>
  ): Promise<void> {
    const startTime = performance.now()

    try {
      await subscriber.callback(event.data)
      subscriber.lastTriggered = new Date()

      // Remove one-time subscribers
      if (subscriber.options.once) {
        this.unsubscribe(subscriber.id)
      }
    } catch (error) {
      console.error(`Error in event subscriber ${subscriber.id}:`, error)

      // Emit error event
      this.emit('error', {
        subscriberId: subscriber.id,
        eventType: event.type,
        error: error instanceof Error ? error.message : 'Unknown error',
        timestamp: new Date()
      }, 'eventBus')
    } finally {
      const processingTime = performance.now() - startTime
      this.updateProcessingMetrics(processingTime)
    }
  }

  private updateMetrics(event: Event): void {
    this.metrics.totalEvents++
    this.metrics.eventsByType[event.type] = (this.metrics.eventsByType[event.type] || 0) + 1
    this.metrics.lastEventTime = event.timestamp
  }

  private updateProcessingMetrics(processingTime: number): void {
    const currentAvg = this.metrics.averageProcessingTime
    const totalEvents = this.metrics.totalEvents

    this.metrics.averageProcessingTime =
      (currentAvg * (totalEvents - 1) + processingTime) / totalEvents
  }

  private addToHistory(event: Event): void {
    this.eventHistory.push(event)

    // Keep only last 1000 events to prevent memory issues
    if (this.eventHistory.length > 1000) {
      this.eventHistory = this.eventHistory.slice(-1000)
    }
  }

  private generateSubscriberId(): string {
    return `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private generateEventId(): string {
    return `evt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }
}

// Predefined event types for the application
export const EventTypes = {
  // Workspace events
  WORKSPACE_CREATED: 'workspace:created',
  WORKSPACE_UPDATED: 'workspace:updated',
  WORKSPACE_DELETED: 'workspace:deleted',
  WORKSPACE_ANALYZED: 'workspace:analyzed',

  // Document events
  DOCUMENT_PROCESSED: 'document:processed',
  DOCUMENT_INDEXED: 'document:indexed',
  DOCUMENT_COMPARED: 'document:compared',
  DOCUMENT_GENERATED: 'document:generated',

  // AI events
  AI_RESPONSE_RECEIVED: 'ai:response:received',
  AI_MODEL_CHANGED: 'ai:model:changed',
  AI_ERROR: 'ai:error',

  // Search events
  SEARCH_COMPLETED: 'search:completed',
  INDEX_UPDATED: 'index:updated',

  // System events
  SYSTEM_ERROR: 'system:error',
  SYSTEM_WARNING: 'system:warning',
  SYSTEM_INFO: 'system:info',

  // Progress events
  PROGRESS_UPDATED: 'progress:updated',
  TASK_COMPLETED: 'task:completed',
  TASK_FAILED: 'task:failed'
} as const

// Export singleton instance
export const eventBus = EventBus.getInstance()

// Create typed emitters for common events
export const workspaceEvents = eventBus.createTypedEmitter('workspace')
export const documentEvents = eventBus.createTypedEmitter('document')
export const aiEvents = eventBus.createTypedEmitter('ai')
export const searchEvents = eventBus.createTypedEmitter('search')
export const systemEvents = eventBus.createTypedEmitter('system')
export const progressEvents = eventBus.createTypedEmitter('progress')