import { useAppStore } from "../../lib/store";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Moon, Sun, Monitor } from "lucide-react";

export function SettingsPage() {
  const { theme, setTheme } = useAppStore();

  return (
    <div className="p-6 space-y-6">
      <h2 className="text-2xl font-bold">Ajustes</h2>

      <Card>
        <CardHeader>
          <CardTitle>Tema visual</CardTitle>
        </CardHeader>
        <CardContent className="flex gap-2">
          <Button
            variant={theme === "dark" ? "default" : "outline"}
            size="sm"
            onClick={() => setTheme("dark")}
          >
            <Moon className="h-4 w-4 mr-1" /> Oscuro
          </Button>
          <Button
            variant={theme === "light" ? "default" : "outline"}
            size="sm"
            onClick={() => setTheme("light")}
          >
            <Sun className="h-4 w-4 mr-1" /> Claro
          </Button>
          <Button
            variant={theme === "system" ? "default" : "outline"}
            size="sm"
            onClick={() => setTheme("system")}
          >
            <Monitor className="h-4 w-4 mr-1" /> Sistema
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Acerca de ClearTool</CardTitle>
        </CardHeader>
        <CardContent className="space-y-1 text-sm text-muted-foreground">
          <p>Versión 0.1.0</p>
          <p>Aplicación de limpieza y optimización para Windows 11</p>
          <p className="font-mono text-xs">Stack: Tauri 2.x + Rust + React + TypeScript</p>
        </CardContent>
      </Card>
    </div>
  );
}
