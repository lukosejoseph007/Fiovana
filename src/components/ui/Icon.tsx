import React from 'react'
import { Icons } from '../../assets/icons/utils'
import type { IconProps } from '../../assets/icons/types'

type IconName = Exclude<keyof typeof Icons, 'getDocumentTypeIcon'>

export interface IconComponentProps extends IconProps {
  name: IconName
  size?: number | string
  color?: string
  className?: string
  'aria-label'?: string
}

export const Icon: React.FC<IconComponentProps> = ({
  name,
  size = 16,
  color = 'currentColor',
  className = '',
  style,
  ...props
}) => {
  const IconComponent = Icons[name] as React.FC<IconProps>

  if (!IconComponent || typeof IconComponent !== 'function') {
    console.warn(`Icon "${name}" not found`)
    return null
  }

  return <IconComponent size={size} color={color} className={className} style={style} {...props} />
}

export default Icon
