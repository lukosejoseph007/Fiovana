import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react'
import * as d3 from 'd3'
import { colors, spacing, borderRadius, shadows, typography } from '../../styles/tokens'
import { relationshipService } from '../../services/relationshipService'
import type {
  GraphNode,
  GraphEdge,
  RelationshipGraph,
  RelationshipCluster,
} from '../../types/relationship'

interface KnowledgeGraphProps {
  workspaceId: string
  selectedDocumentId?: string
  onNodeClick?: (nodeId: string) => void
  onClusterClick?: (cluster: RelationshipCluster) => void
  height?: number
  enableClustering?: boolean
  showLabels?: boolean
  strengthThreshold?: number
}

interface D3Node extends GraphNode {
  x?: number
  y?: number
  vx?: number
  vy?: number
  fx?: number | null
  fy?: number | null
  index?: number
}

interface D3Link extends GraphEdge {
  source: D3Node | string | number
  target: D3Node | string | number
}

interface SimulationNode extends d3.SimulationNodeDatum {
  id: string
  documentId: string
  label: string
  type: string
  properties: Record<string, unknown>
  position?: { x: number; y: number; z?: number }
  style?: {
    color?: string
    size?: number
    shape?: string
    icon?: string
  }
}

export const KnowledgeGraph: React.FC<KnowledgeGraphProps> = ({
  workspaceId,
  selectedDocumentId,
  onNodeClick,
  onClusterClick,
  height = 600,
  enableClustering = true,
  showLabels = true,
  strengthThreshold = 0.3,
}) => {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [graphData, setGraphData] = useState<RelationshipGraph | null>(null)
  const [clusters, setClusters] = useState<RelationshipCluster[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedNode, setSelectedNode] = useState<string | null>(selectedDocumentId || null)
  const [hoveredNode, setHoveredNode] = useState<string | null>(null)
  const [transform, setTransform] = useState({ k: 1, x: 0, y: 0 })

  // Load graph data
  const loadGraphData = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      // Build relationship graph
      const graphResponse = await relationshipService.buildRelationshipGraph(workspaceId, true)

      if (graphResponse.success && graphResponse.data) {
        setGraphData(graphResponse.data as RelationshipGraph)
      } else {
        throw new Error('Failed to load graph data')
      }

      // Load clusters if enabled
      if (enableClustering) {
        const clustersResponse = await relationshipService.identifyDocumentClusters(workspaceId)
        if (clustersResponse.success && clustersResponse.data) {
          setClusters(clustersResponse.data as RelationshipCluster[])
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load graph')
      console.error('Graph loading error:', err)
    } finally {
      setLoading(false)
    }
  }, [workspaceId, enableClustering])

  useEffect(() => {
    loadGraphData()
  }, [loadGraphData])

  // Update selected node when prop changes
  useEffect(() => {
    if (selectedDocumentId) {
      setSelectedNode(selectedDocumentId)
    }
  }, [selectedDocumentId])

  // Get node color based on type and state
  const getNodeColor = useCallback(
    (node: D3Node): string => {
      if (node.style?.color) return node.style.color

      // Selected node
      if (node.documentId === selectedNode) return colors.accent.ai

      // Hovered node
      if (node.documentId === hoveredNode) return colors.accent.semantic

      // Node type colors
      switch (node.type) {
        case 'document':
          return colors.accent.info
        case 'concept':
          return colors.accent.semantic
        case 'procedure':
          return colors.accent.success
        case 'reference':
          return colors.accent.warning
        default:
          return colors.accent.info
      }
    },
    [selectedNode, hoveredNode]
  )

  // Get node size based on properties
  const getNodeSize = useCallback((node: D3Node): number => {
    if (node.style?.size) return node.style.size

    // Size based on connections or importance
    const importance = (node.properties.importance as number) || 1
    const baseSize = 5
    return baseSize + importance * 3
  }, [])

  // Get edge color based on relationship type
  const getEdgeColor = useCallback((edge: GraphEdge): string => {
    if (edge.style?.color) return edge.style.color

    // Strength-based color
    if (edge.weight > 0.8) return colors.accent.success
    if (edge.weight > 0.5) return colors.accent.semantic
    if (edge.weight > 0.3) return colors.accent.info
    return colors.border.subtle
  }, [])

  // Get edge width based on weight
  const getEdgeWidth = useCallback((edge: GraphEdge): number => {
    if (edge.style?.width) return edge.style.width
    return 1 + edge.weight * 2
  }, [])

  // Memoized simulation setup
  const simulationSetup = useMemo(() => {
    if (!graphData || !svgRef.current || !containerRef.current) return null

    const width = containerRef.current.clientWidth
    const nodes: D3Node[] = graphData.nodes.map(n => ({
      ...n,
      x: n.position?.x,
      y: n.position?.y,
    }))

    const links: D3Link[] = graphData.edges
      .filter(e => e.weight >= strengthThreshold)
      .map(e => ({
        ...e,
        source: e.sourceId,
        target: e.targetId,
      }))

    return { nodes, links, width }
  }, [graphData, strengthThreshold])

  // D3 visualization
  useEffect(() => {
    if (!simulationSetup) return

    const { nodes, links, width } = simulationSetup
    const vizHeight = height
    const svg = d3.select(svgRef.current)

    // Clear previous content
    svg.selectAll('*').remove()

    // Create main group for zoom/pan
    const g = svg.append('g').attr('class', 'main-group')

    // Define arrow markers for directed edges
    svg
      .append('defs')
      .append('marker')
      .attr('id', 'arrowhead')
      .attr('viewBox', '-0 -5 10 10')
      .attr('refX', 15)
      .attr('refY', 0)
      .attr('orient', 'auto')
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .append('svg:path')
      .attr('d', 'M 0,-5 L 10,0 L 0,5')
      .attr('fill', colors.border.medium)

    // Create force simulation
    const simulation = d3
      .forceSimulation<SimulationNode>(nodes as SimulationNode[])
      .force(
        'link',
        d3
          .forceLink<SimulationNode, d3.SimulationLinkDatum<SimulationNode>>(
            links as d3.SimulationLinkDatum<SimulationNode>[]
          )
          .id(d => d.id)
          .distance(d => {
            const link = d as unknown as GraphEdge
            return 100 / (link.weight || 0.5)
          })
          .strength(d => {
            const link = d as unknown as GraphEdge
            return link.weight
          })
      )
      .force('charge', d3.forceManyBody<SimulationNode>().strength(-300))
      .force('center', d3.forceCenter<SimulationNode>(width / 2, vizHeight / 2))
      .force('collision', d3.forceCollide<SimulationNode>().radius(20))

    // Draw clusters (background circles)
    if (enableClustering && clusters.length > 0) {
      const clusterGroup = g.append('g').attr('class', 'clusters')

      clusters.forEach(cluster => {
        const clusterNodes = nodes.filter(n => cluster.documents.includes(n.documentId))
        if (clusterNodes.length === 0) return

        const centroid = {
          x: d3.mean(clusterNodes, n => n.x || 0) || 0,
          y: d3.mean(clusterNodes, n => n.y || 0) || 0,
        }

        const radius = Math.max(
          ...clusterNodes.map(n => {
            const dx = (n.x || 0) - centroid.x
            const dy = (n.y || 0) - centroid.y
            return Math.sqrt(dx * dx + dy * dy)
          })
        )

        clusterGroup
          .append('circle')
          .attr('cx', centroid.x)
          .attr('cy', centroid.y)
          .attr('r', radius + 30)
          .attr('fill', colors.glass.white5)
          .attr('stroke', colors.border.subtle)
          .attr('stroke-width', 1)
          .attr('stroke-dasharray', '5,5')
          .style('cursor', 'pointer')
          .on('click', () => {
            onClusterClick?.(cluster)
          })
      })
    }

    // Draw edges
    const link = g
      .append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', d => getEdgeColor(d as unknown as GraphEdge))
      .attr('stroke-width', d => getEdgeWidth(d as unknown as GraphEdge))
      .attr('stroke-opacity', 0.6)
      .attr('marker-end', 'url(#arrowhead)')

    // Draw nodes
    const node = g
      .append('g')
      .attr('class', 'nodes')
      .selectAll('circle')
      .data(nodes)
      .join('circle')
      .attr('r', d => getNodeSize(d))
      .attr('fill', d => getNodeColor(d))
      .attr('stroke', colors.border.strong)
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .call(
        d3
          .drag<SVGCircleElement, D3Node>()
          .on('start', (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart()
            d.fx = d.x
            d.fy = d.y
          })
          .on('drag', (event, d) => {
            d.fx = event.x
            d.fy = event.y
          })
          .on('end', (event, d) => {
            if (!event.active) simulation.alphaTarget(0)
            d.fx = null
            d.fy = null
          }) as (
          selection: d3.Selection<SVGCircleElement | d3.BaseType, D3Node, SVGGElement, unknown>
        ) => void
      )
      .on('click', (_event, d) => {
        setSelectedNode(d.documentId)
        onNodeClick?.(d.documentId)
      })
      .on('mouseenter', (_event, d) => {
        setHoveredNode(d.documentId)

        // Pulse animation on hover
        d3.select(_event.currentTarget as SVGCircleElement)
          .transition()
          .duration(200)
          .attr('r', getNodeSize(d) * 1.5)
      })
      .on('mouseleave', (_event, d) => {
        setHoveredNode(null)

        d3.select(_event.currentTarget as SVGCircleElement)
          .transition()
          .duration(200)
          .attr('r', getNodeSize(d))
      })

    // Add labels if enabled
    if (showLabels) {
      const labels = g
        .append('g')
        .attr('class', 'labels')
        .selectAll('text')
        .data(nodes)
        .join('text')
        .text(d => d.label)
        .attr('font-size', typography.fontSize.xs)
        .attr('font-family', typography.fonts.sans.join(', '))
        .attr('fill', colors.text.primary)
        .attr('text-anchor', 'middle')
        .attr('dy', d => getNodeSize(d) + 12)
        .style('pointer-events', 'none')
        .style('user-select', 'none')

      simulation.on('tick', () => {
        link
          .attr('x1', d => (typeof d.source === 'object' ? d.source.x || 0 : 0))
          .attr('y1', d => (typeof d.source === 'object' ? d.source.y || 0 : 0))
          .attr('x2', d => (typeof d.target === 'object' ? d.target.x || 0 : 0))
          .attr('y2', d => (typeof d.target === 'object' ? d.target.y || 0 : 0))

        node.attr('cx', d => d.x || 0).attr('cy', d => d.y || 0)

        labels.attr('x', d => d.x || 0).attr('y', d => d.y || 0)
      })
    } else {
      simulation.on('tick', () => {
        link
          .attr('x1', d => (typeof d.source === 'object' ? d.source.x || 0 : 0))
          .attr('y1', d => (typeof d.source === 'object' ? d.source.y || 0 : 0))
          .attr('x2', d => (typeof d.target === 'object' ? d.target.x || 0 : 0))
          .attr('y2', d => (typeof d.target === 'object' ? d.target.y || 0 : 0))

        node.attr('cx', d => d.x || 0).attr('cy', d => d.y || 0)
      })
    }

    // Zoom behavior with momentum scrolling
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', event => {
        g.attr('transform', event.transform)
        setTransform({ k: event.transform.k, x: event.transform.x, y: event.transform.y })
      })

    svg.call(
      zoom as (selection: d3.Selection<SVGSVGElement | null, unknown, null, undefined>) => void
    )

    // Initial zoom to fit
    const bounds = g.node()?.getBBox()
    if (bounds) {
      const fullWidth = bounds.width
      const fullHeight = bounds.height
      const midX = bounds.x + fullWidth / 2
      const midY = bounds.y + fullHeight / 2

      const scale = Math.min(width / fullWidth, vizHeight / fullHeight) * 0.9
      const translate = [width / 2 - scale * midX, vizHeight / 2 - scale * midY]

      const initialTransform = d3.zoomIdentity
        .translate(translate[0] || 0, translate[1] || 0)
        .scale(scale)
      svg
        .transition()
        .duration(750)
        .call(transition => {
          zoom.transform(
            transition as unknown as d3.Selection<SVGSVGElement, unknown, null, undefined>,
            initialTransform
          )
        })
    }

    return () => {
      simulation.stop()
    }
  }, [
    simulationSetup,
    selectedNode,
    hoveredNode,
    clusters,
    enableClustering,
    showLabels,
    onNodeClick,
    onClusterClick,
    getNodeColor,
    getNodeSize,
    getEdgeColor,
    getEdgeWidth,
    height,
  ])

  if (loading) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: `${height}px`,
          background: colors.surface.secondary,
          borderRadius: borderRadius.lg,
          color: colors.text.secondary,
          fontFamily: typography.fonts.sans.join(', '),
        }}
      >
        <div style={{ textAlign: 'center' }}>
          <div
            style={{
              width: '40px',
              height: '40px',
              border: `3px solid ${colors.border.subtle}`,
              borderTop: `3px solid ${colors.accent.ai}`,
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
              margin: '0 auto 12px',
            }}
          />
          <p style={{ fontSize: typography.fontSize.sm }}>Loading knowledge graph...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: `${height}px`,
          background: colors.surface.secondary,
          borderRadius: borderRadius.lg,
          color: colors.accent.alert,
          fontFamily: typography.fonts.sans.join(', '),
          padding: spacing[6],
          textAlign: 'center',
        }}
      >
        <div>
          <p style={{ fontSize: typography.fontSize.base, marginBottom: spacing[2] }}>
            Failed to load knowledge graph
          </p>
          <p style={{ fontSize: typography.fontSize.sm, color: colors.text.tertiary }}>{error}</p>
          <button
            onClick={loadGraphData}
            style={{
              marginTop: spacing[4],
              padding: `${spacing[2]} ${spacing[4]}`,
              background: colors.accent.ai,
              color: colors.surface.primary,
              border: 'none',
              borderRadius: borderRadius.base,
              cursor: 'pointer',
              fontSize: typography.fontSize.sm,
              fontFamily: typography.fonts.sans.join(', '),
            }}
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  if (!graphData || graphData.nodes.length === 0) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: `${height}px`,
          background: colors.surface.secondary,
          borderRadius: borderRadius.lg,
          color: colors.text.secondary,
          fontFamily: typography.fonts.sans.join(', '),
        }}
      >
        <p>No relationships found in workspace</p>
      </div>
    )
  }

  return (
    <div
      style={{
        position: 'relative',
        background: colors.surface.secondary,
        borderRadius: borderRadius.lg,
        overflow: 'hidden',
        boxShadow: shadows.base,
      }}
    >
      {/* Graph Controls */}
      <div
        style={{
          position: 'absolute',
          top: spacing[3],
          right: spacing[3],
          display: 'flex',
          gap: spacing[2],
          zIndex: 10,
        }}
      >
        <button
          onClick={() => {
            if (svgRef.current) {
              const svg = d3.select(svgRef.current)
              const resetZoom = d3.zoom<SVGSVGElement, unknown>()
              svg
                .transition()
                .duration(750)
                .call(transition => {
                  resetZoom.transform(
                    transition as unknown as d3.Selection<SVGSVGElement, unknown, null, undefined>,
                    d3.zoomIdentity
                  )
                })
            }
          }}
          style={{
            padding: spacing[2],
            background: colors.surface.tertiary,
            border: `1px solid ${colors.border.subtle}`,
            borderRadius: borderRadius.base,
            color: colors.text.primary,
            cursor: 'pointer',
            fontSize: typography.fontSize.xs,
            fontFamily: typography.fonts.sans.join(', '),
          }}
          title="Reset zoom"
        >
          Reset
        </button>
        <button
          onClick={loadGraphData}
          style={{
            padding: spacing[2],
            background: colors.surface.tertiary,
            border: `1px solid ${colors.border.subtle}`,
            borderRadius: borderRadius.base,
            color: colors.text.primary,
            cursor: 'pointer',
            fontSize: typography.fontSize.xs,
            fontFamily: typography.fonts.sans.join(', '),
          }}
          title="Refresh graph"
        >
          Refresh
        </button>
      </div>

      {/* Graph Stats */}
      <div
        style={{
          position: 'absolute',
          bottom: spacing[3],
          left: spacing[3],
          padding: spacing[2],
          background: colors.glass.white10,
          backdropFilter: 'blur(10px)',
          borderRadius: borderRadius.base,
          fontSize: typography.fontSize.xs,
          color: colors.text.secondary,
          fontFamily: typography.fonts.sans.join(', '),
          zIndex: 10,
        }}
      >
        <div>
          Nodes: {graphData.nodes.length} | Edges: {graphData.edges.length} | Zoom:{' '}
          {transform.k.toFixed(2)}x
        </div>
      </div>

      {/* SVG Container */}
      <div ref={containerRef} style={{ width: '100%', height: `${height}px` }}>
        <svg
          ref={svgRef}
          width="100%"
          height={height}
          style={{
            display: 'block',
            background: colors.surface.primary,
          }}
        />
      </div>

      {/* Add keyframes animation for loading spinner */}
      <style>
        {`
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
        `}
      </style>
    </div>
  )
}
