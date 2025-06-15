import React, {useState, useEffect} from 'react';
import {
  Modal,
  View,
  Text,
  TextInput,
  TouchableOpacity,
  Animated,
  Dimensions,
  Easing,
  Keyboard,
  TouchableWithoutFeedback,
} from 'react-native';
import {useSetAtom} from 'jotai';
import {styles} from './ServersScreen.styles';
import {Server, addServerAtom} from '../atoms/servers';

interface AddServerModalProps {
  visible: boolean;
  onClose: () => void;
}

export const AddServerModal: React.FC<AddServerModalProps> = ({
  visible,
  onClose,
}) => {
  const [name, setName] = useState('MC Complex');
  const [address, setAddress] = useState('mps.mc-complex.com');
  const [port, setPort] = useState('19132');
  const [errors, setErrors] = useState<{
    name?: string;
    address?: string;
    port?: string;
  }>({});
  const translateY = new Animated.Value(Dimensions.get('window').height);
  const opacityAnim = new Animated.Value(0);
  const addServer = useSetAtom(addServerAtom);

  const validateForm = () => {
    const newErrors: typeof errors = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!address.trim()) {
      newErrors.address = 'Address is required';
    }

    const portNum = parseInt(port, 10);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      newErrors.port = 'Port must be between 1 and 65535';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = () => {
    if (!validateForm()) {
      return;
    }

    const newServer: Omit<Server, 'status' | 'data'> = {
      id: Date.now().toString(),
      name: name.trim(),
      address: address.trim(),
      port: port.trim(),
    };

    addServer(newServer);
    console.log('Saving server:', newServer);

    // Reset form and close modal
    setName('MC Complex');
    setAddress('mps.mc-complex.com');
    setPort('19132');
    setErrors({});
    onClose();
  };

  const handleDismiss = () => {
    Keyboard.dismiss();
    onClose();
  };

  useEffect(() => {
    if (visible) {
      Animated.parallel([
        Animated.timing(translateY, {
          toValue: 0,
          duration: 300,
          useNativeDriver: true,
          easing: Easing.bezier(0.25, 0.1, 0.25, 1),
        }),
        Animated.timing(opacityAnim, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
      ]).start();
    } else {
      Animated.parallel([
        Animated.timing(translateY, {
          toValue: Dimensions.get('window').height,
          duration: 250,
          useNativeDriver: true,
          easing: Easing.bezier(0.25, 0.1, 0.25, 1),
        }),
        Animated.timing(opacityAnim, {
          toValue: 0,
          duration: 250,
          useNativeDriver: true,
        }),
      ]).start();
    }
  }, [visible]);

  return (
    <Modal visible={visible} transparent animationType="none">
      <TouchableWithoutFeedback onPress={handleDismiss}>
        <Animated.View
          style={[
            styles.modalOverlay,
            {
              opacity: opacityAnim,
            },
          ]}>
          <TouchableWithoutFeedback>
            <Animated.View
              style={[
                styles.modalContainer,
                {
                  transform: [{translateY}],
                },
              ]}>
              <Text style={styles.modalTitle}>Add Server</Text>
              <Text style={styles.modalLabel}>Name</Text>
              <TextInput
                style={[styles.input, errors.name && styles.inputError]}
                placeholder="Example Server"
                placeholderTextColor="#aaa"
                value={name}
                onChangeText={text => {
                  setName(text);
                  if (errors.name) {
                    setErrors(prev => ({...prev, name: undefined}));
                  }
                }}
                returnKeyType="next"
              />
              {errors.name && (
                <Text style={styles.errorText}>{errors.name}</Text>
              )}
              <Text style={styles.modalLabel}>Address</Text>
              <TextInput
                style={[styles.input, errors.address && styles.inputError]}
                placeholder="mc.example.com"
                placeholderTextColor="#aaa"
                value={address}
                onChangeText={text => {
                  setAddress(text);
                  if (errors.address) {
                    setErrors(prev => ({...prev, address: undefined}));
                  }
                }}
                returnKeyType="next"
                autoCapitalize="none"
              />
              {errors.address && (
                <Text style={styles.errorText}>{errors.address}</Text>
              )}
              <Text style={styles.modalLabel}>Port</Text>
              <TextInput
                style={[styles.input, errors.port && styles.inputError]}
                placeholder="19132"
                placeholderTextColor="#aaa"
                value={port}
                onChangeText={text => {
                  setPort(text);
                  if (errors.port) {
                    setErrors(prev => ({...prev, port: undefined}));
                  }
                }}
                keyboardType="numeric"
                returnKeyType="done"
                onSubmitEditing={handleSave}
              />
              {errors.port && (
                <Text style={styles.errorText}>{errors.port}</Text>
              )}
              <View style={styles.modalButtonRow}>
                <TouchableOpacity
                  style={styles.saveButton}
                  onPress={handleSave}>
                  <Text style={styles.saveButtonText}>Save</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  style={styles.cancelButton}
                  onPress={handleDismiss}>
                  <Text style={styles.cancelButtonText}>Cancel</Text>
                </TouchableOpacity>
              </View>
            </Animated.View>
          </TouchableWithoutFeedback>
        </Animated.View>
      </TouchableWithoutFeedback>
    </Modal>
  );
};
