import React from 'react';
import type { LayoutContextType } from './AppShell';

export const LayoutContext = React.createContext<LayoutContextType | undefined>(undefined);

export const useLayout = () => {
  const context = React.useContext(LayoutContext);
  if (!context) {
    throw new Error('Layout components must be used within AppShell');
  }
  return context;
};