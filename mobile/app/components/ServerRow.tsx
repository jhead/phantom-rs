import React, {useState} from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Alert,
  Modal,
  Dimensions,
} from 'react-native';
import {Server, ServerStatus} from '../services/serverState';
import {styles} from './ServersScreen.styles';
import {
  parseMinecraftColors,
  MinecraftTextSegment,
} from '../utils/minecraftColors';

const StatusIndicator: React.FC<{status: ServerStatus}> = ({status}) => {
  const getStatusColor = () => {
    switch (status) {
      case 'online':
        return '#4CAF50';
      case 'offline':
        return '#F44336';
      case 'connecting':
        return '#FFC107';
      case 'starting':
        return '#2196F3';
    }
  };

  return (
    <View
      style={[
        styles.statusIndicator,
        {
          backgroundColor: getStatusColor(),
        },
      ]}
    />
  );
};

const ServerIcon: React.FC<{icon?: string}> = ({icon}) => {
  if (!icon) {
    return (
      <View style={styles.serverIconPlaceholder}>
        <View style={styles.serverIconInner}>
          <View style={styles.serverIconLine} />
          <View style={[styles.serverIconLine, {width: '60%'}]} />
          <View style={[styles.serverIconLine, {width: '80%'}]} />
        </View>
      </View>
    );
  }

  return (
    <View style={styles.serverIcon}>
      <View style={styles.serverIconInner}>
        <View style={styles.serverIconLine} />
        <View style={[styles.serverIconLine, {width: '60%'}]} />
        <View style={[styles.serverIconLine, {width: '80%'}]} />
      </View>
    </View>
  );
};

const PlayerCount: React.FC<{
  players?: number;
  maxPlayers?: number;
}> = ({players, maxPlayers}) => {
  if (players === undefined || maxPlayers === undefined) {
    return (
      <View style={styles.playerCountPlaceholder}>
        <View style={styles.playerCountSkeleton} />
      </View>
    );
  }

  return (
    <Text style={styles.playerCount}>
      {players}/{maxPlayers} players
    </Text>
  );
};

const MOTDLine: React.FC<{text: string}> = ({text}) => {
  const segments = parseMinecraftColors(text);

  return (
    <Text style={styles.motdLine} numberOfLines={1}>
      {segments.map((segment, index) => (
        <Text
          key={index}
          style={{
            color: segment.color || '#CCCCCC',
            fontWeight: segment.bold ? 'bold' : 'normal',
            fontStyle: segment.italic ? 'italic' : 'normal',
            textDecorationLine: segment.underline
              ? 'underline'
              : segment.strikethrough
              ? 'line-through'
              : 'none',
          }}>
          {segment.text}
        </Text>
      ))}
    </Text>
  );
};

const MOTD: React.FC<{motd?: [string, string]}> = ({motd}) => {
  if (!motd) {
    return (
      <View style={styles.motdPlaceholder}>
        <View style={styles.motdSkeleton} />
        <View style={[styles.motdSkeleton, {width: '70%'}]} />
      </View>
    );
  }

  return (
    <View style={styles.motdContainer}>
      <MOTDLine text={motd[0]} />
      <MOTDLine text={motd[1]} />
    </View>
  );
};

const LastUpdated: React.FC<{timestamp?: number}> = ({timestamp}) => {
  const [relativeTime, setRelativeTime] = React.useState<string>('');

  React.useEffect(() => {
    if (!timestamp) return;

    const updateTime = () => {
      const now = Date.now();
      const diff = now - timestamp;
      const seconds = Math.floor(diff / 1000);

      if (seconds < 60) {
        setRelativeTime(`${seconds}s ago`);
      } else {
        const minutes = Math.floor(seconds / 60);
        if (minutes < 60) {
          setRelativeTime(`${minutes}m ago`);
        } else {
          const hours = Math.floor(minutes / 60);
          if (hours < 24) {
            setRelativeTime(`${hours}h ago`);
          } else {
            const days = Math.floor(hours / 24);
            setRelativeTime(`${days}d ago`);
          }
        }
      }
    };

    // Update immediately
    updateTime();

    // Then update every second
    const interval = setInterval(updateTime, 1000);

    return () => clearInterval(interval);
  }, [timestamp]);

  if (!timestamp) {
    return (
      <View style={styles.lastUpdatedPlaceholder}>
        <View style={styles.lastUpdatedSkeleton} />
      </View>
    );
  }

  return <Text style={styles.lastUpdated}>Updated {relativeTime}</Text>;
};

interface ServerRowProps {
  server: Server;
  onPress?: () => void;
  onDelete?: () => void;
  onEdit?: () => void;
}

export const ServerRow: React.FC<ServerRowProps> = ({
  server,
  onPress,
  onDelete,
  onEdit,
}) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  console.log(
    `Rendering server row for ${server.name} with status ${server.status}`,
  );

  const handleMoreOptions = () => {
    console.log(`More options pressed for server: ${server.name}`);
    setIsMenuOpen(true);
  };

  const handleDelete = () => {
    setIsMenuOpen(false);
    Alert.alert(
      'Delete Server',
      `Are you sure you want to delete ${server.name}?`,
      [
        {
          text: 'Cancel',
          style: 'cancel',
        },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: () => {
            if (onDelete) {
              console.log(`Delete confirmed for server: ${server.name}`);
              onDelete();
            }
          },
        },
      ],
      {cancelable: true},
    );
  };

  const handleEdit = () => {
    setIsMenuOpen(false);
    if (onEdit) {
      console.log(`Edit requested for server: ${server.name}`);
      onEdit();
    }
  };

  const handleCancel = () => {
    setIsMenuOpen(false);
  };

  if (isMenuOpen) {
    return (
      <View style={styles.serverItem}>
        <View style={styles.serverHeader}>
          <ServerIcon icon={server.data?.icon} />
          <View style={styles.serverInfo}>
            <Text style={styles.serverName}>{server.name}</Text>
            <Text style={styles.serverAddress}>
              {server.address}:{server.port}
            </Text>
          </View>
          <StatusIndicator status={server.status} />
          <TouchableOpacity
            style={styles.moreOptionsButton}
            onPress={handleCancel}
            hitSlop={{top: 10, bottom: 10, left: 10, right: 10}}>
            <Text style={styles.moreOptionsText}>⋮</Text>
          </TouchableOpacity>
        </View>
        <View style={styles.menuRow}>
          {onEdit && (
            <TouchableOpacity style={styles.menuButton} onPress={handleEdit}>
              <Text style={styles.menuButtonText}>Edit</Text>
            </TouchableOpacity>
          )}
          {onDelete && (
            <TouchableOpacity
              style={[styles.menuButton, styles.menuButtonDestructive]}
              onPress={handleDelete}>
              <Text style={styles.menuButtonTextDestructive}>Delete</Text>
            </TouchableOpacity>
          )}
          <TouchableOpacity style={styles.menuButton} onPress={handleCancel}>
            <Text style={styles.menuButtonText}>Cancel</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <TouchableOpacity style={styles.serverItem} onPress={onPress}>
      <View style={styles.serverHeader}>
        <ServerIcon icon={server.data?.icon} />
        <View style={styles.serverInfo}>
          <Text style={styles.serverName}>{server.name}</Text>
          <Text style={styles.serverAddress}>
            {server.address}:{server.port}
          </Text>
        </View>
        <StatusIndicator status={server.status} />
        {(onDelete || onEdit) && (
          <TouchableOpacity
            style={styles.moreOptionsButton}
            onPress={handleMoreOptions}
            hitSlop={{top: 10, bottom: 10, left: 10, right: 10}}>
            <Text style={styles.moreOptionsText}>⋮</Text>
          </TouchableOpacity>
        )}
      </View>
      <View style={styles.serverDetails}>
        <View style={styles.serverStats}>
          <PlayerCount
            players={server.data?.players}
            maxPlayers={server.data?.maxPlayers}
          />
          <LastUpdated timestamp={server.data?.lastPing} />
        </View>
        <MOTD motd={server.data?.motd} />
      </View>
    </TouchableOpacity>
  );
};
