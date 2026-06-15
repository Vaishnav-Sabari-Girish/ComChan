#include <zephyr/kernel.h>
#include <zephyr/bluetooth/bluetooth.h>
#include <zephyr/bluetooth/hci.h>
#include <zephyr/bluetooth/conn.h>
#include <zephyr/bluetooth/uuid.h>
#include <zephyr/bluetooth/gatt.h>
#include <stdio.h>

#define DEVICE_NAME CONFIG_BT_DEVICE_NAME
#define DEVICE_NAME_LEN (sizeof(DEVICE_NAME) - 1)

/* Nordic UART Service (NUS) UUID Definitions */
#define BT_UUID_NUS_VAL \
    BT_UUID_128_ENCODE(0x6e400001, 0xb5a3, 0xf393, 0xe0a9, 0xe50e24dcca9e)
#define BT_UUID_NUS_TX_VAL \
    BT_UUID_128_ENCODE(0x6e400003, 0xb5a3, 0xf393, 0xe0a9, 0xe50e24dcca9e)
#define BT_UUID_NUS_RX_VAL \
    BT_UUID_128_ENCODE(0x6e400002, 0xb5a3, 0xf393, 0xe0a9, 0xe50e24dcca9e)

static struct bt_uuid_128 nus_uuid    = BT_UUID_INIT_128(BT_UUID_NUS_VAL);
static struct bt_uuid_128 nus_tx_uuid = BT_UUID_INIT_128(BT_UUID_NUS_TX_VAL);
static struct bt_uuid_128 nus_rx_uuid = BT_UUID_INIT_128(BT_UUID_NUS_RX_VAL);

static uint8_t tx_notify_enabled;

static void nus_ccc_cfg_changed(const struct bt_gatt_attr *attr, uint16_t value)
{
    tx_notify_enabled = (value == BT_GATT_CCC_NOTIFY);
    printk("NUS TX notifications %s\n", tx_notify_enabled ? "enabled" : "disabled");
}

static ssize_t write_rx(struct bt_conn *conn, const struct bt_gatt_attr *attr,
                        const void *buf, uint16_t len, uint16_t offset, uint8_t flags)
{
    printk("Received data on NUS RX (len: %d)\n", len);
    return len;
}

/* Expose the GATT Service Table */
BT_GATT_SERVICE_DEFINE(nus_svc,
    BT_GATT_PRIMARY_SERVICE(&nus_uuid),
    
    /* TX Characteristic (Peripheral to Central) */
    BT_GATT_CHARACTERISTIC(&nus_tx_uuid.uuid, 
                           BT_GATT_CHRC_NOTIFY, 
                           BT_GATT_PERM_NONE, 
                           NULL, NULL, NULL),
    BT_GATT_CCC(nus_ccc_cfg_changed, BT_GATT_PERM_READ | BT_GATT_PERM_WRITE),

    /* RX Characteristic (Central to Peripheral) */
    BT_GATT_CHARACTERISTIC(&nus_rx_uuid.uuid, 
                           BT_GATT_CHRC_WRITE | BT_GATT_CHRC_WRITE_WITHOUT_RESP, 
                           BT_GATT_PERM_WRITE, 
                           NULL, write_rx, NULL),
);

/* Advertising Data Setup */
static const struct bt_data ad[] = {
    BT_DATA_BYTES(BT_DATA_FLAGS, (BT_LE_AD_GENERAL | BT_LE_AD_NO_BREDR)),
    BT_DATA(BT_DATA_NAME_COMPLETE, DEVICE_NAME, DEVICE_NAME_LEN),
};

/* * Secondary scan response data.
 * Fix: Use BT_DATA_UUID128_ALL instead of BT_DATA_UUID128_COMPLETE for newer Zephyr versions.
 */
static const struct bt_data sd[] = {
    BT_DATA_BYTES(BT_DATA_UUID128_ALL, BT_UUID_NUS_VAL),
};

static void connected(struct bt_conn *conn, uint8_t err) {
    if (err) {
        printk("Connection failed (err %u)\n", err);
    } else {
        printk("Device connected successfully!\n");
    }
}

static void disconnected(struct bt_conn *conn, uint8_t reason) {
    printk("Device disconnected (reason %u)\n", reason);
    tx_notify_enabled = 0;
}

BT_CONN_CB_DEFINE(conn_callbacks) = {
    .connected = connected,
    .disconnected = disconnected,
};

int main(void) {
    int err;

    printk("Starting BLE Peripheral with NUS Profile...\n");

    err = bt_enable(NULL);
    if (err) {
        printk("Bluetooth initialization failed (err %d)\n", err);
        return 0;
    }

    printk("Bluetooth initialized.\n");

    err = bt_le_adv_start(BT_LE_ADV_CONN_FAST_1, ad, ARRAY_SIZE(ad), sd, ARRAY_SIZE(sd));
    if (err) {
        printk("Advertising failed to start (err %d)\n", err);
        return 0;
    }

    printk("Advertising successfully started with NUS UUID.\n");

    /* Background loop pushing data to ComChan via notifications */
    while (1) {
        k_sleep(K_MSEC(100)); // Sleep for 100ms (10Hz update rate)
        
        if (tx_notify_enabled) {
            static int angle = 0;
            char msg[64];
            
            // Format dummy stream matching typical ComChan parsing
            snprintf(msg, sizeof(msg), "Pitch: %d, Roll: %d, Yaw: %d\n", 
                     angle, angle / 2, (angle * 2) % 360);
            
            // Capture the error code!
            int err = bt_gatt_notify(NULL, &nus_svc.attrs[2], msg, strlen(msg));
            if (err) {
                printk("Failed to notify (err %d)\n", err);
            }
            
            angle = (angle + 1) % 360;
        }
    }

    return 0;
}
