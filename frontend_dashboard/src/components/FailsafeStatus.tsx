import React from 'react';

export interface FailsafeEvent {
    timestamp: number;
    type: string;
    action: string;
    description: string;
    recovered: boolean;
}

export const FailsafeStatus: React.FC<{ status: FailsafeEvent[] }> = ({ status }) => {
    const critical = status.filter(e => e.action === 'EMERGENCY_LAND' || e.action === 'RETURN_TO_LAUNCH');
    
    if (critical.length > 0) {
        return <div className="critical-failsafe">🚨 FAILSAFE ACTIVE - {critical[0].description}</div>;
    }
    
    return <div className="healthy">✅ Systems Nominal</div>;
};
