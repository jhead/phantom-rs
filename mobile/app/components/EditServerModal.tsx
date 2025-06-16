import React, {useMemo} from 'react';
import {Server} from '../services/serverState';
import {ServerFormModal} from './ServerFormModal';

interface EditServerModalProps {
  visible: boolean;
  onClose: () => void;
  onEdit: (
    serverId: string,
    updates: Partial<Omit<Server, 'id' | 'status' | 'data'>>,
  ) => Promise<void>;
  server: Server | null;
}

export const EditServerModal: React.FC<EditServerModalProps> = ({
  visible,
  onClose,
  onEdit,
  server,
}) => {
  if (!server) {
    return null;
  }

  const handleSubmit = async (
    serverData: Omit<Server, 'status' | 'data' | 'autoStart'>,
  ) => {
    await onEdit(server.id, {
      name: serverData.name,
      address: serverData.address,
      port: serverData.port,
    });
  };

  const initialValues = useMemo(
    () => ({
      name: server.name,
      address: server.address,
      port: server.port,
    }),
    [server.name, server.address, server.port],
  );

  return (
    <ServerFormModal
      visible={visible}
      onClose={onClose}
      onSubmit={handleSubmit}
      initialValues={initialValues}
      title="Edit Server"
      submitButtonText="Update"
    />
  );
};
