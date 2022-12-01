
#include <stdio.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <immintrin.h>
#include <dirent.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <stdint.h>
#include <stddef.h>
#include <pthread.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <stdarg.h>
#include <sys/mman.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
#include <assert.h>
#include <string.h>
#include <sys/mman.h>
#include <immintrin.h>

#include <fpga_pci.h>
#include <fpga_mgmt.h>
#include <utils/lcd.h>

static uint16_t pci_vendor_id = 0x1D0F; /* Amazon PCI Vendor ID */
static uint16_t pci_device_id = 0xF001; /* PCI Device ID preassigned by Amazon for F1 applications */

static uint32_t pci_offset;
static pci_bar_handle_t pci_bar_handle_0;
static pci_bar_handle_t pci_bar_handle_4;
volatile uint32_t* pcie_addr_4;

int check_afi_ready(int slot_id) {
   struct fpga_mgmt_image_info info = {0};
   int rc;

   /* get local image description, contains status, vendor id, and device id. */
   rc = fpga_mgmt_describe_local_image(slot_id, &info,0);
   fail_on(rc, out, "Unable to get AFI information from slot %d. Are you running as root?",slot_id);

   /* check to see if the slot is ready */
   if (info.status != FPGA_STATUS_LOADED) {
     rc = 1;
     fail_on(rc, out, "AFI in Slot %d is not in READY state !", slot_id);
   }
/*
   printf("AFI PCI  Vendor ID: 0x%x, Device ID 0x%x\n",
          info.spec.map[FPGA_APP_PF].vendor_id,
          info.spec.map[FPGA_APP_PF].device_id);
*/
   /* confirm that the AFI that we expect is in fact loaded */
   if (info.spec.map[FPGA_APP_PF].vendor_id != pci_vendor_id ||
       info.spec.map[FPGA_APP_PF].device_id != pci_device_id) {
     printf("AFI does not show expected PCI vendor id and device ID. If the AFI "
            "was just loaded, it might need a rescan. Rescanning now.\n");

     rc = fpga_pci_rescan_slot_app_pfs(slot_id);
     fail_on(rc, out, "Unable to update PF for slot %d",slot_id);
     /* get local image description, contains status, vendor id, and device id. */
     rc = fpga_mgmt_describe_local_image(slot_id, &info,0);
     fail_on(rc, out, "Unable to get AFI information from slot %d",slot_id);

     printf("AFI PCI  Vendor ID: 0x%x, Device ID 0x%x\n",
            info.spec.map[FPGA_APP_PF].vendor_id,
            info.spec.map[FPGA_APP_PF].device_id);

     /* confirm that the AFI that we expect is in fact loaded after rescan */
     if (info.spec.map[FPGA_APP_PF].vendor_id != pci_vendor_id ||
         info.spec.map[FPGA_APP_PF].device_id != pci_device_id) {
       rc = 1;
       fail_on(rc, out, "The PCI vendor id and device of the loaded AFI are not "
               "the expected values.");
     }
   }

   return rc;
out:
   return 1;
}

int init_f1(int slot_id, int off)
{
    int rc;
    pci_bar_handle_0 = PCI_BAR_HANDLE_INIT;
    pci_bar_handle_4 = PCI_BAR_HANDLE_INIT;
    pci_offset = off;

    rc = fpga_mgmt_init();
    fail_on(rc, out, "Unable to initialize the fpga_mgmt library");

    rc = check_afi_ready(slot_id);
    fail_on(rc, out, "AFI not ready");

    rc = fpga_pci_attach(slot_id, FPGA_APP_PF, APP_PF_BAR0, 0, &pci_bar_handle_0);
    fail_on(rc, out, "Unable to attach to the AFI on slot id %d", slot_id);
    rc = fpga_pci_attach(slot_id, FPGA_APP_PF, APP_PF_BAR4, BURST_CAPABLE, &pci_bar_handle_4);
    fail_on(rc, out, "Unable to attach to the AFI on slot id %d", slot_id);

    fpga_pci_get_address(pci_bar_handle_4, 0, 1024*1024, (void**)&pcie_addr_4);

    return rc;
out:
    if (pci_bar_handle_0 >= 0) {
        rc = fpga_pci_detach(pci_bar_handle_0);
        if (rc) {
            printf("Failure while detaching from the fpga.\n");
        }
    }
    return (rc != 0 ? 1 : 0);
}

uint32_t read_32_f1(uint32_t addr)
{
    int rc;
    uint32_t value;
    addr |= (1L << 30L);
    fpga_pci_poke(pci_bar_handle_0, pci_offset+0, addr);
    rc = fpga_pci_peek(pci_bar_handle_0, pci_offset+0, &value);
    fail_on(rc, out, "Unable to read from the fpga !");
out:
    return value;
}

void write_32_f1(uint32_t addr, uint32_t v)
{
    addr |= (2L << 30L);
    fpga_pci_poke(pci_bar_handle_0, pci_offset+4, v);
    fpga_pci_poke(pci_bar_handle_0, pci_offset+0, addr);
}

void write_256(uint64_t off, void* buf)
{
    uint32_t* data = (uint32_t*)buf;
    volatile uint32_t* addr = pcie_addr_4;
    addr += (off >> 2);

    __m256i v;
    v = _mm256_load_si256((__m256i*)data);
    _mm256_stream_si256((__m256i*)(addr), v);
}

void write_512_f1(uint64_t off, uint8_t* buf)
{
    write_256(off+ 0, &buf[ 0]);
    write_256(off+32, &buf[32]);
}

void write_flush()
{
    _mm_sfence();
}
