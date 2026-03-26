import {
  Tabs as RsTabs,
  Tab,
} from '@rspress/core/theme';
import React from 'react';
import type { ReactNode, ReactElement } from 'react';

const icons: Record<string, string> = {
  Leptos: '/images/frameworks/leptos.png',
  Dioxus: '/images/frameworks/dioxus.png',
  Yew: '/images/frameworks/yew.png',
  'Vanilla JS': '/images/frameworks/js.png',
};

function enrichLabel(label: string): ReactNode {
  const icon = icons[label];
  if (!icon) return label;
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '6px',
      }}
    >
      <img
        src={icon}
        width={16}
        height={16}
        alt=""
        style={{ display: 'block' }}
      />
      {label}
    </span>
  );
}

function Tabs({
  children,
  ...props
}: {
  children: ReactNode;
  [key: string]: any;
}) {
  const enhanced = React.Children.map(children, (child) => {
    if (!React.isValidElement(child)) return child;
    const label = (child.props as any).label;
    if (typeof label === 'string' && icons[label]) {
      return React.cloneElement(
        child as ReactElement<any>,
        { label: enrichLabel(label) },
      );
    }
    return child;
  });
  return <RsTabs {...props}>{enhanced}</RsTabs>;
}

export { Tabs, Tab };
