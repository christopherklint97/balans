import { Toaster as Sonner, type ToasterProps } from 'sonner';
import { useTheme } from '@/hooks/use-theme';

export function Toaster(props: ToasterProps) {
  const { theme } = useTheme();

  return (
    <Sonner
      theme={theme as ToasterProps['theme']}
      className="toaster group"
      position="top-right"
      richColors
      closeButton
      style={
        {
          '--normal-bg': 'var(--color-popover)',
          '--normal-text': 'var(--color-popover-foreground)',
          '--normal-border': 'var(--color-border)',
        } as React.CSSProperties
      }
      {...props}
    />
  );
}
